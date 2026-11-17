package xyz.fractal_tess.oxevu;

import android.content.Context;
import com.getcapacitor.JSObject;
import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;

@CapacitorPlugin(name = "OxevuServer")
public class ServerPlugin extends Plugin {
    static String server(Context context) {
        return context.getSharedPreferences("oxevu-server", Context.MODE_PRIVATE).getString("url", null);
    }

    @PluginMethod public void getServer(PluginCall call) {
        JSObject result = new JSObject();
        result.put("url", server(getContext()) == null ? org.json.JSONObject.NULL : server(getContext()));
        call.resolve(result);
    }

    @PluginMethod public void setServer(PluginCall call) {
        try {
            String origin = ServerOrigin.validate(call.getString("url", ""));
            getContext().getSharedPreferences("oxevu-server", Context.MODE_PRIVATE)
                .edit().putString("url", origin).apply();
            call.resolve();
        } catch (java.net.URISyntaxException error) {
            call.reject("Invalid server address");
        }
    }
}
