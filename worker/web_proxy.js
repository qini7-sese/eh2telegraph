/*
    Cloudflare workers telegraph proxy.
    Deploy and set `KEY` variable in browser.
*/

addEventListener('fetch', event => {
    event.respondWith(handleRequest(event.request))
})

const RESPONSE_HEADERS = {
    "Server": "web-proxy",
};

async function handleRequest(request) {
    // validate request key
    if (request.headers.get("X-Authorization") != KEY) {
        return new Response(null, {
            status: 401,
            headers: RESPONSE_HEADERS
        });
    }

    // read original url
    var url = request.headers.get("X-Forwarded-For");
    if (url == null || url == "") {
        return new Response(null, {
            status: 400,
            headers: RESPONSE_HEADERS
        });
    }

    // construct new url and request
    var req;
    if (request.body && request.method != 'GET' && request.method != 'HEAD') {
        req = new Request(new URL(url), {
            method: request.method,
            headers: request.headers,
            body: request.body
        });
    } else {
        req = new Request(new URL(url), {
            method: request.method,
            headers: request.headers,
        });
    }

    // remove headers
    req.headers.delete("X-Authorization");
    req.headers.delete("X-Forwarded-For");
    req.headers.delete("CF-Connecting-IP");
    req.headers.delete("CF-Worker");
    req.headers.delete("CF-EW-Via");

    // send request
    var result = await fetch(req);
    return result;
}