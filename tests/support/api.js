// The server's API as the page tests (tests/page.rs) have it answer: every document of the page
// runs this first, and the page's requests under /api/ are answered here, so that none reaches
// the server, or a novel site through it. Other requests (the page's files, its manifest and
// icons) go to the server.
{
  const json = (body) =>
    new Response(JSON.stringify(body), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  const answer = (url, method) => {
    if (url.includes("/api/auth/me")) {
      return json({ email: "guest" });
    }
    if (url.includes("/api/favorites") && method === "GET") {
      return json([
        { type: "narou", id: "n1234ab", title: "テスト小説", page: 10, read: 3, novelupdated_at: null },
      ]);
    }
    if (url.includes("/ranking")) {
      return json({
        総合: [
          { id: "n9999zz", title: "ランキング透け確認用の長いタイトル文字列を並べる", page: 50, noveltype: 1 },
          { id: "n8888yy", title: "二件目の小説タイトルでスクロールする", page: 12, noveltype: 1 },
        ],
      });
    }
    if (url.includes("/detail")) {
      return json({ title: "テスト小説", synopsis: "あらすじ", page: 10 });
    }
    if (url.includes("/pages/")) {
      return json({ html: "<p>本文です。スクロール用の段落。</p>".repeat(20) });
    }
    return json([]);
  };
  const fetched = window.fetch.bind(window);
  window.fetch = (input, options = {}) => {
    const request = new Request(input, options);
    const url = new URL(request.url, location.href);
    if (url.origin === location.origin && url.pathname.includes("/api/")) {
      return Promise.resolve(answer(url.href, request.method));
    }
    return fetched(input, options);
  };
}
