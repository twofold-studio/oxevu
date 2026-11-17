import * as generated from "./generated/sdk.gen";
import { sdkClient, sdkData } from "./sdk";

export type Provider = keyof Omit<
  import("./generated/types.gen").IntegrationStatuses,
  "studio_images"
>;
export type { ProviderStatus, ProviderUpdate, IntegrationEnvelope } from "./generated/types.gen";

export const integrationsApi = {
  get: () => sdkData(generated.integrationsGet({ client: sdkClient(), throwOnError: true })),
  update: (body: import("./generated/types.gen").IntegrationUpdates) =>
    sdkData(generated.integrationsPut({ client: sdkClient(), throwOnError: true, body })),
};
