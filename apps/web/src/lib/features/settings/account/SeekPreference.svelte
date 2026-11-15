<script lang="ts">
  import { readSeekSeconds, saveSeekSeconds, validSeekSeconds } from "$lib/playback/seek-preference";
  import NumberField from "../playback/NumberField.svelte";

  let seconds = $state(readSeekSeconds());
  let error = $state<string | null>(null);

  function save() {
    error = !validSeekSeconds(seconds)
      ? "Enter a whole number from 1 to 60."
      : saveSeekSeconds(seconds) ? null : "This browser could not save the setting.";
  }
</script>

<div onchange={save}>
  <NumberField id="seek-seconds" label="Seek interval" unit="seconds" min={1} max={60}
    bind:value={seconds} {error}
    description="Left/Right arrows and skip buttons. Saved automatically for this browser. Try 5, 7, 10, 12 or 15 seconds." />
</div>
