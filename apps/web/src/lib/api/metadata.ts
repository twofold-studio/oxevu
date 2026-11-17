import * as generated from "./generated/sdk.gen";
import { sdkClient, sdkData } from "./sdk";

export type { MetadataProvider, MetadataResult } from "./generated/types.gen";
import type { MetadataProvider, MetadataResult } from "./generated/types.gen";

export const metadataApi = {
  search: (id: string, provider: MetadataProvider, query: string) =>
    sdkData(
      generated.metadataSearch({
        client: sdkClient(),
        throwOnError: true,
        path: { id },
        query: { provider, query },
      }),
    ),
  match: (id: string, result: Pick<MetadataResult, "provider" | "id">) =>
    sdkData(
      generated.metadataMatch({
        client: sdkClient(),
        throwOnError: true,
        path: { id },
        body: { provider: result.provider, id: result.id },
      }),
    ),
};
