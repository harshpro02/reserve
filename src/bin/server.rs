use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use deadpool_postgres::{Manager, Pool};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio_postgres::{NoTls, Transaction};

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;
fn error(code: StatusCode, message: &str) -> (StatusCode, Json<Value>) {
    (code, Json(json!({"error":message})))
}
fn internal(e: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    eprintln!("database: {e}");
    error(StatusCode::SERVICE_UNAVAILABLE, "Database unavailable")
}
async fn db(pool: &Pool) -> Result<deadpool_postgres::Object, deadpool_postgres::PoolError> {
    pool.get().await
}
async fn now(tx: &Transaction<'_>) -> Result<i64, tokio_postgres::Error> {
    Ok(tx
        .query_one(
            "SELECT floor(extract(epoch FROM clock_timestamp()))::bigint",
            &[],
        )
        .await?
        .get(0))
}
async fn expire(tx: &Transaction<'_>, sku: &str, at: i64) -> Result<(), tokio_postgres::Error> {
    tx.execute("UPDATE reserve_holds SET status='expired' WHERE sku=$1 AND status='held' AND expires_at <= $2", &[&sku,&at]).await?;
    Ok(())
}
fn hold(row: &tokio_postgres::Row) -> Value {
    json!({"id":row.get::<_,String>("id"),"sku":row.get::<_,String>("sku"),"quantity":row.get::<_,i32>("quantity"),"status":row.get::<_,String>("status"),"expires_at":row.get::<_,i64>("expires_at")})
}

#[derive(Deserialize)]
struct Create {
    sku: String,
    quantity: i32,
    ttl_seconds: i32,
    idempotency_key: String,
}

async fn create(State(url): State<Pool>, Json(input): Json<Create>) -> ApiResult {
    if input.quantity <= 0
        || !(1..=3600).contains(&input.ttl_seconds)
        || input.idempotency_key.is_empty()
        || input.idempotency_key.len() > 128
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Positive quantity, TTL 1–3600, and a key of 1–128 bytes required",
        ));
    }
    let mut c = db(&url).await.map_err(internal)?;
    let tx = c.transaction().await.map_err(internal)?;
    let stock = tx
        .query_opt(
            "SELECT on_hand FROM reserve_stock WHERE sku=$1 FOR UPDATE",
            &[&input.sku],
        )
        .await
        .map_err(internal)?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Unknown SKU"))?;
    let at = now(&tx).await.map_err(internal)?;
    expire(&tx, &input.sku, at).await.map_err(internal)?;
    if let Some(old) = tx
        .query_opt(
            "SELECT * FROM reserve_holds WHERE sku=$1 AND request_key=$2",
            &[&input.sku, &input.idempotency_key],
        )
        .await
        .map_err(internal)?
    {
        if old.get::<_, i32>("quantity") != input.quantity
            || old.get::<_, i32>("ttl") != input.ttl_seconds
        {
            return Err(error(
                StatusCode::CONFLICT,
                "Key already used with a different request",
            ));
        }
        let result = hold(&old);
        tx.commit().await.map_err(internal)?;
        return Ok(Json(json!({"reservation":result,"replayed":true})));
    }
    let held:i64=tx.query_one("SELECT COALESCE(sum(quantity),0)::bigint FROM reserve_holds WHERE sku=$1 AND status='held'", &[&input.sku]).await.map_err(internal)?.get(0);
    if i64::from(stock.get::<_, i32>(0)) - held < i64::from(input.quantity) {
        tx.commit().await.map_err(internal)?;
        return Err(error(StatusCode::CONFLICT, "Not enough available stock"));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let deadline = at + i64::from(input.ttl_seconds);
    let row=tx.query_one("INSERT INTO reserve_holds(id,sku,quantity,status,expires_at,request_key,ttl) VALUES($1,$2,$3,'held',$4,$5,$6) RETURNING *", &[&id,&input.sku,&input.quantity,&deadline,&input.idempotency_key,&input.ttl_seconds]).await.map_err(internal)?;
    let result = hold(&row);
    tx.commit().await.map_err(internal)?;
    Ok(Json(json!({"reservation":result,"replayed":false})))
}

async fn transition(url: Pool, id: String, action: &str) -> ApiResult {
    let mut c = db(&url).await.map_err(internal)?;
    let tx = c.transaction().await.map_err(internal)?;
    let initial = tx
        .query_opt("SELECT sku FROM reserve_holds WHERE id=$1", &[&id])
        .await
        .map_err(internal)?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Unknown reservation"))?;
    let sku: String = initial.get(0);
    tx.query_one(
        "SELECT sku FROM reserve_stock WHERE sku=$1 FOR UPDATE",
        &[&sku],
    )
    .await
    .map_err(internal)?;
    expire(&tx, &sku, now(&tx).await.map_err(internal)?)
        .await
        .map_err(internal)?;
    let row = tx
        .query_one("SELECT * FROM reserve_holds WHERE id=$1", &[&id])
        .await
        .map_err(internal)?;
    let status: String = row.get("status");
    if status != action && status != "held" {
        tx.commit().await.map_err(internal)?;
        return Err(error(
            StatusCode::CONFLICT,
            "Reservation is no longer active",
        ));
    }
    if status == "held" {
        if action == "confirmed" {
            tx.execute(
                "UPDATE reserve_stock SET on_hand=on_hand-$2 WHERE sku=$1",
                &[&sku, &row.get::<_, i32>("quantity")],
            )
            .await
            .map_err(internal)?;
        }
        tx.execute(
            "UPDATE reserve_holds SET status=$2 WHERE id=$1",
            &[&id, &action],
        )
        .await
        .map_err(internal)?;
    }
    let result = hold(
        &tx.query_one("SELECT * FROM reserve_holds WHERE id=$1", &[&id])
            .await
            .map_err(internal)?,
    );
    tx.commit().await.map_err(internal)?;
    Ok(Json(result))
}
async fn confirm(State(url): State<Pool>, Path(id): Path<String>) -> ApiResult {
    transition(url, id, "confirmed").await
}
async fn cancel(State(url): State<Pool>, Path(id): Path<String>) -> ApiResult {
    transition(url, id, "cancelled").await
}
async fn snapshot(State(url): State<Pool>) -> ApiResult {
    let mut c = db(&url).await.map_err(internal)?;
    let tx = c.transaction().await.map_err(internal)?;
    let stocks = tx
        .query(
            "SELECT sku,name,on_hand FROM reserve_stock ORDER BY sku FOR UPDATE",
            &[],
        )
        .await
        .map_err(internal)?;
    let at = now(&tx).await.map_err(internal)?;
    let mut products = vec![];
    for row in stocks {
        let sku: String = row.get(0);
        expire(&tx, &sku, at).await.map_err(internal)?;
        let held:i64=tx.query_one("SELECT COALESCE(sum(quantity),0)::bigint FROM reserve_holds WHERE sku=$1 AND status='held'", &[&sku]).await.map_err(internal)?.get(0);
        let on_hand: i32 = row.get(2);
        products.push(json!({"sku":sku,"name":row.get::<_,String>(1),"on_hand":on_hand,"held":held,"available":i64::from(on_hand)-held}));
    }
    let reservations: Vec<_> = tx
        .query(
            "SELECT * FROM reserve_holds ORDER BY expires_at DESC LIMIT 100",
            &[],
        )
        .await
        .map_err(internal)?
        .iter()
        .map(hold)
        .collect();
    tx.commit().await.map_err(internal)?;
    Ok(Json(
        json!({"products":products,"reservations":reservations,"server_time":at}),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "host=127.0.0.1 user=portfolio password=portfolio_local dbname=portfolio".into()
    });
    let manager = Manager::new(url.parse::<tokio_postgres::Config>()?, NoTls);
    let url = Pool::builder(manager).max_size(16).build()?;
    let c = db(&url).await?;
    c.batch_execute(include_str!("../../migrations/001.sql"))
        .await?;
    drop(c);
    let app = Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../../web/index.html")) }),
        )
        .route("/health", get(|| async { "ok" }))
        .route("/api/state", get(snapshot))
        .route("/api/reservations", post(create))
        .route("/api/reservations/{id}/confirm", post(confirm))
        .route("/api/reservations/{id}/cancel", post(cancel))
        .with_state(url);
    let address = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:4101".into());
    println!("Reserve http://{address}");
    axum::serve(tokio::net::TcpListener::bind(address).await?, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
