import unittest

import httpx

from pennykite import PennyKiteBudgetExceeded, PennyKiteClient, PennyKiteDenied, PennyKiteLoopDetected


class PennyKiteClientTests(unittest.TestCase):
    def test_import_and_context_manager(self):
        with PennyKiteClient(session_id="session-1", transport=httpx.MockTransport(lambda request: httpx.Response(200))) as client:
            self.assertEqual(client.session_id, "session-1")

    def test_get_forwards_path_query_and_session_header(self):
        seen = []

        def handler(request):
            seen.append(request)
            return httpx.Response(200, json={"ok": True})

        client = PennyKiteClient(
            session_id="trade-1",
            proxy_url="http://proxy.test",
            transport=httpx.MockTransport(handler),
        )
        response = client.get("https://api.example.test/v1/casts?fid=123")

        self.assertEqual(response.json(), {"ok": True})
        self.assertEqual(str(seen[0].url), "http://proxy.test/v1/casts?fid=123")
        self.assertEqual(seen[0].headers["x-pennykite-session"], "trade-1")

    def test_post_forwards_json_body(self):
        def handler(request):
            self.assertEqual(request.content, b'{"side":"buy"}')
            return httpx.Response(200, json={"ok": True})

        client = PennyKiteClient(session_id="trade-2", transport=httpx.MockTransport(handler))
        response = client.post("/orders", json={"side": "buy"})

        self.assertEqual(response.status_code, 200)

    def test_budget_denial_raises_typed_exception(self):
        client = PennyKiteClient(
            session_id="trade-3",
            transport=httpx.MockTransport(
                lambda request: httpx.Response(
                    402,
                    json={
                        "verdict": "deny_budget",
                        "reason": "session budget exceeded",
                        "decision_id": "decision-1",
                        "decision_hash": "0xabc",
                        "estimated_cost_usd": 0.25,
                    },
                )
            ),
        )

        with self.assertRaises(PennyKiteBudgetExceeded) as raised:
            client.get("/expensive")
        self.assertEqual(raised.exception.verdict, "deny_budget")
        self.assertEqual(raised.exception.decision_id, "decision-1")
        self.assertEqual(raised.exception.estimated_cost_usd, 0.25)

    def test_loop_denial_raises_typed_exception(self):
        client = PennyKiteClient(
            session_id="trade-4",
            transport=httpx.MockTransport(
                lambda request: httpx.Response(
                    402,
                    json={"verdict": "deny_loop", "reason": "loop detected"},
                )
            ),
        )

        with self.assertRaises(PennyKiteLoopDetected):
            client.get("/same-call")

    def test_other_pennykite_denial_raises_base_denial(self):
        client = PennyKiteClient(
            session_id="trade-5",
            transport=httpx.MockTransport(
                lambda request: httpx.Response(
                    402,
                    json={"verdict": "deny_network", "reason": "network blocked"},
                )
            ),
        )

        with self.assertRaises(PennyKiteDenied):
            client.get("/blocked-network")

    def test_non_pennykite_402_is_returned(self):
        client = PennyKiteClient(
            session_id="trade-6",
            transport=httpx.MockTransport(lambda request: httpx.Response(402, text="payment required")),
        )

        self.assertEqual(client.get("/raw-402").status_code, 402)


if __name__ == "__main__":
    unittest.main()
