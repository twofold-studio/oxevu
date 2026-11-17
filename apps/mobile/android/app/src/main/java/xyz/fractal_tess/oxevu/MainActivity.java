package xyz.fractal_tess.oxevu;

import com.getcapacitor.BridgeActivity;
import android.os.Bundle;

public class MainActivity extends BridgeActivity {
    @Override public void onCreate(Bundle savedInstanceState) {
        androidx.appcompat.app.AppCompatDelegate.setDefaultNightMode(androidx.appcompat.app.AppCompatDelegate.MODE_NIGHT_YES);
        registerPlugin(ServerPlugin.class);
        super.onCreate(savedInstanceState);
        bridge.setWebViewClient(new MediaClient(bridge));
    }
}
