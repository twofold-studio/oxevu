# Full-episode quality check

On 2026-11-15, Arcane S1E1 was converted in full using the highest current
player preset: 1080p, an 8 Mbps ceiling, CUDA decode/scale to NV12 and
H264 NVENC preset p4. Output remained 1920×816 at 24 fps. No playback settings
or media files were changed. The test used Matroska output without audio;
video encoder, scaling and forced two-second IDR settings matched playback.
It does not test HLS delivery, audio or browser rendering.

The 43:25 output was 617,971,877 bytes, averaging 1.90 Mbps. The command sets
`-maxrate 8000k -bufsize 16000k`, but no explicit NVENC target bitrate or CQ.
The UI preset is a ceiling, not a guarantee of encoder quality or output bitrate.

Both streams were aligned with `setpts=PTS-STARTPTS`, converted to 8-bit
YUV420 and compared with libvmaf, sampling every sixth frame across the full
episode. Mean VMAF was 95.78, lowest sampled frame 72.05 at 11:35.25. VMAF is
a perceptual similarity score, not a percentage of retained image detail.
The comparison includes the source's existing compression and conversion from
10-bit to 8-bit; it does not isolate encoder loss from pixel-format conversion.

Lowest ten-second average ranges included 00:40–00:50 (85.62), 02:50–03:00
(88.80), and 01:00–01:10 (90.17). Matching source/output frames were inspected
at 00:45 and 11:35.25. The latter visibly loses fine texture in floor panels,
boxes and background surfaces. The former contains dark red gradients and
texture, which also differ after conversion.

Next experiment: set an explicit NVENC VBR target/quality policy and compare
the same ranges before changing defaults. Increasing a ceiling alone does not
guarantee improvement. This test did not compare alternative encoder policies.
