import type { DiscoverTitle, MonitoredTitle } from "$lib/api/acquisition-types";
import { emptyScope } from "./format";

export function newSeriesSelection(title: DiscoverTitle): MonitoredTitle {
  return {
    title,
    units: [],
    monitor: {
      enabled: true,
      automatic: false,
      library_id: null,
      root: null,
      profile_id: "default",
      scope: emptyScope(),
    },
  };
}
