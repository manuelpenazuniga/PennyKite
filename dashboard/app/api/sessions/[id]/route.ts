import { NextResponse } from "next/server";
import { proxyUrl } from "@/lib/proxy";
import type { DecisionFeed } from "@/lib/types";

export const dynamic = "force-dynamic";

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ id: string }> },
) {
  const { id } = await params;

  try {
    const res = await fetch(proxyUrl(`/api/sessions/${encodeURIComponent(id)}`), {
      cache: "no-store",
    });
    if (!res.ok) {
      return NextResponse.json(
        { error: "proxy_session_error", reason: `proxy returned HTTP ${res.status}` },
        { status: res.status === 404 ? 404 : 502 },
      );
    }

    const data = (await res.json()) as DecisionFeed;
    return NextResponse.json(data, {
      headers: { "cache-control": "no-store" },
    });
  } catch (e) {
    return NextResponse.json(
      {
        error: "proxy_unreachable",
        reason: e instanceof Error ? e.message : "fetch failed",
      },
      { status: 502 },
    );
  }
}
