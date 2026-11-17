import type { CapacitorConfig } from "@capacitor/cli";

const config: CapacitorConfig = {
  appId: "xyz.fractal_tess.oxevu",
  appName: "Oxevu",
  webDir: "dist",
  android: { path: "../mobile/android" },
  ios: { path: "../mobile/ios" },
  // Bundle the shared UI. Never load a remote page into the native bridge.
  loggingBehavior: "none",
  plugins: { SystemBars: { style: "DARK" } },
};

export default config;
