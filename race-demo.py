"""Run inside Ubuntu WSL with the local Reserve server running."""
import concurrent.futures
import json
import sys
import threading
import time
import uuid
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from runpy import run_path
helpers = run_path(str(Path(__file__).resolve().parents[1] / "verify-all.py"))
ReserveTests, percentile, reserve, sql = (helpers[name] for name in ("ReserveTests", "percentile", "reserve", "sql"))


def run(case, sku_count, requests, workers):
    skus = ["RACE-" + uuid.uuid4().hex for _ in range(sku_count)]
    quoted = ",".join(f"'{sku}'" for sku in skus)
    units = 1 if case == "last_item" else 25 // sku_count
    values = ",".join(f"('{sku}','Race fixture',{units})" for sku in skus)
    barrier = threading.Barrier(workers, timeout=15)

    def attempt(index):
        if index < workers:
            barrier.wait()
        started = time.perf_counter()
        port = 4101 + 4 * (index % 2)
        status, _ = reserve(skus[(index // 2) % sku_count], 1, port=port)
        return status, (time.perf_counter() - started) * 1000, port

    sql(f"INSERT INTO reserve_stock VALUES {values};")
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
            results = list(pool.map(attempt, range(requests)))
        accepted = sum(code == 200 for code, _, _ in results)
        conflicts = sum(code == 409 for code, _, _ in results)
        expected = 1 if case == "last_item" else 25
        assert accepted == expected and accepted + conflicts == requests, results
        return dict(case=case, requests=requests, workers=workers, sku_count=sku_count,
                    accepted=accepted, conflicts=conflicts,
                    p95_ms=round(percentile([ms for _, ms, _ in results], .95), 3),
                    outcomes=[dict(port=port,status=code) for code, _, port in results] if case=="last_item" else None)
    finally:
        sql(f"DELETE FROM reserve_holds WHERE sku IN ({quoted}); DELETE FROM reserve_stock WHERE sku IN ({quoted});")


if __name__ == "__main__":
    ReserveTests.setUpClass()
    try:
        report = [run("last_item", 1, 2, 2)]
        for _ in range(3):
            report.extend([run("hot_sku", 1, 100, 16), run("five_skus", 5, 100, 16)])
        print(json.dumps(report, indent=2))
    finally:
        ReserveTests.tearDownClass()
