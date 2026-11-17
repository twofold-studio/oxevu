package xyz.fractal_tess.oxevu;

import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebView;
import com.getcapacitor.Bridge;
import com.getcapacitor.BridgeWebViewClient;
import java.io.ByteArrayInputStream;
import java.io.FilterInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.Semaphore;

/** Stream authenticated media at the bundled UI origin, without buffering files or exposing cookies. */
final class MediaClient extends BridgeWebViewClient {
    private final Bridge owner;
    private final Semaphore readers = new Semaphore(16);

    MediaClient(Bridge bridge) { super(bridge); owner = bridge; }

    @Override public WebResourceResponse shouldInterceptRequest(WebView view, WebResourceRequest request) {
        String path = request.getUrl().getEncodedPath();
        boolean local = java.util.Objects.equals(request.getUrl().getAuthority(), android.net.Uri.parse(owner.getLocalUrl()).getAuthority());
        if (!local || path == null || !path.startsWith("/api/v1/")) {
            return super.shouldInterceptRequest(view, request);
        }
        String server = ServerPlugin.server(owner.getContext());
        if (server == null || request.isForMainFrame() || !request.getMethod().equals("GET")) return unavailable();
        for (Map.Entry<String, String> header : request.getRequestHeaders().entrySet()) {
            if (header.getKey().equalsIgnoreCase("Upgrade-Insecure-Requests")
                    || (header.getKey().equalsIgnoreCase("Sec-Fetch-Dest")
                    && (header.getValue().equals("document") || header.getValue().equals("iframe")))) {
                return unavailable();
            }
        }
        if (!readers.tryAcquire()) return unavailable();
        HttpURLConnection connection = null;
        try {
            String query = request.getUrl().getEncodedQuery();
            URL remote = new URL(server + path + (query == null ? "" : "?" + query));
            connection = (HttpURLConnection) remote.openConnection();
            connection.setInstanceFollowRedirects(false);
            connection.setConnectTimeout(10000);
            connection.setReadTimeout(30000);
            connection.setRequestProperty("Accept-Encoding", "identity");
            String cookie = android.webkit.CookieManager.getInstance().getCookie(server);
            if (cookie != null) connection.setRequestProperty("Cookie", cookie);
            for (Map.Entry<String, String> header : request.getRequestHeaders().entrySet()) {
                if (header.getKey().equalsIgnoreCase("Range") || header.getKey().equalsIgnoreCase("If-Range")) {
                    connection.setRequestProperty(header.getKey(), header.getValue());
                }
            }
            int status = connection.getResponseCode();
            if (status >= 300 && status < 400) throw new IOException("Redirect refused");
            Map<String, String> headers = new HashMap<>();
            for (String name : new String[]{"Content-Type", "Content-Length", "Content-Range", "Accept-Ranges", "Cache-Control"}) {
                String value = connection.getHeaderField(name);
                if (value != null) headers.put(name, value);
            }
            // Do not keep authenticated media in a shared WebView cache across server/account changes.
            headers.put("Cache-Control", "no-store");
            String mime = connection.getContentType();
            if (mime == null) mime = "application/octet-stream";
            HttpURLConnection active = connection;
            InputStream body = status >= 400 ? connection.getErrorStream() : connection.getInputStream();
            if (body == null) body = new ByteArrayInputStream(new byte[0]);
            InputStream stream = new FilterInputStream(body) {
                private boolean closed;
                @Override public synchronized void close() throws IOException {
                    if (closed) return;
                    closed = true;
                    try { super.close(); } finally { active.disconnect(); readers.release(); }
                }
            };
            return new WebResourceResponse(mime.split(";")[0], null, status, "Oxevu", headers, stream);
        } catch (IOException | IllegalArgumentException error) {
            if (connection != null) connection.disconnect();
            readers.release();
            return unavailable();
        }
    }

    private static WebResourceResponse unavailable() {
        return new WebResourceResponse("text/plain", "UTF-8", 503, "Unavailable", java.util.Collections.singletonMap("Cache-Control", "no-store"), new ByteArrayInputStream(new byte[0]));
    }
}
