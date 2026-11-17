package xyz.fractal_tess.oxevu;

import java.net.URI;
import java.net.URISyntaxException;

final class ServerOrigin {
    private ServerOrigin() {}

    static String validate(String input) throws URISyntaxException {
        URI uri = new URI(input);
        if (!("http".equals(uri.getScheme()) || "https".equals(uri.getScheme()))
                || uri.getHost() == null || uri.getRawUserInfo() != null
                || uri.getRawQuery() != null || uri.getRawFragment() != null
                || (uri.getRawPath() != null && !uri.getRawPath().isEmpty())
                || uri.getPort() == 0 || uri.getPort() > 65535) {
            throw new URISyntaxException(input, "Expected an HTTP or HTTPS origin");
        }
        return uri.toString();
    }
}
