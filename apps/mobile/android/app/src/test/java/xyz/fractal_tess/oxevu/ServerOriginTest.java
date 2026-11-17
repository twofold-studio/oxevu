package xyz.fractal_tess.oxevu;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import org.junit.Test;

public class ServerOriginTest {
    @Test public void acceptsLocalMeshAndSecureOrigins() throws Exception {
        for (String origin : new String[]{"http://192.168.0.2:8096", "http://vd.netbird.cloud:8096", "https://media.example.com", "http://[::1]:8096"}) {
            assertEquals(origin, ServerOrigin.validate(origin));
        }
    }

    @Test public void rejectsUnsafeNativeBridgeInput() {
        for (String origin : new String[]{"file:///etc/passwd", "https://user:secret@example.com", "https://example.com/api", "https://example.com?token=x", "http://example.com:0", "http://example.com:70000", "javascript:alert(1)", "https://example.com/#x"}) {
            assertThrows(java.net.URISyntaxException.class, () -> ServerOrigin.validate(origin));
        }
    }
}
