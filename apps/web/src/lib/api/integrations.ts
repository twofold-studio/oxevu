import { request } from "./client";

export type Provider = "tmdb" | "omdb" | "opensubtitles";
export interface ProviderStatus {
  enabled: boolean;
  configured: boolean;
  api_key_set: boolean;
  username?: string;
  password_set?: boolean;
}
export interface IntegrationEnvelope {
  integrations: Record<Provider, ProviderStatus> & { studio_images: { enabled: boolean } };
}
export interface ProviderUpdate {
  enabled?: boolean;
  api_key?: string;
  username?: string;
  password?: string;
  clear_credentials?: boolean;
}
export const integrationsApi = {
  get: () => request<IntegrationEnvelope>("/integrations"),
  update: (body: Partial<Record<Provider | "studio_images", ProviderUpdate>>) =>
    request<IntegrationEnvelope>("/integrations", { method: "PUT", body }),
};
