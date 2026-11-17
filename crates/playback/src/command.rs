use tokio::process::Command;
pub fn configure(
    mut cmd: Command,
    input: &std::path::Path,
    dir: &std::path::Path,
    start: f64,
    method: &str,
    s: &oxevu_config::Settings,
    decode: bool,
) -> Command {
    cmd.arg("-ss").arg(start.to_string());
    if decode {
        cmd.args(["-hwaccel", "cuda", "-hwaccel_device"])
            .arg(s.gpu_index.to_string())
            .args(["-hwaccel_output_format", "cuda"]);
    }
    cmd.arg("-i")
        .arg(input)
        .args(["-map", "0:v:0", "-map", "0:a:0?", "-sn", "-dn"]);
    if method == "remux" {
        cmd.args(["-c:v", "copy"]);
    } else {
        let filter = if decode {
            format!(
                "scale_cuda=w=-2:h='trunc(min(ih,{})/2)*2':format=nv12{}",
                s.max_height,
                if method == "nvenc" {
                    ""
                } else {
                    ",hwdownload,format=nv12"
                }
            )
        } else {
            format!(
                "scale=-2:'trunc(min(ih,{})/2)*2',format=yuv420p",
                s.max_height
            )
        };
        cmd.args([
            "-vf",
            &filter,
            "-c:v",
            if method == "nvenc" {
                "h264_nvenc"
            } else {
                "libx264"
            },
        ]);
        if method == "nvenc" {
            cmd.arg("-gpu").arg(s.gpu_index.to_string()).args([
                "-preset",
                "p4",
                "-forced-idr",
                "1",
            ]);
        } else {
            cmd.args(["-preset", "veryfast", "-crf"])
                .arg(s.cpu_crf.to_string());
        }
        cmd.args([
            "-maxrate",
            &format!("{}k", (s.max_video_bitrate_mbps * 1000.) as u64),
            "-bufsize",
            &format!("{}k", (s.max_video_bitrate_mbps * 2000.) as u64),
            "-force_key_frames",
            "expr:gte(t,n_forced*2)",
        ]);
    }
    cmd.args([
        "-threads",
        "2",
        "-c:a",
        "aac",
        "-ac",
        "2",
        "-b:a",
        "128k",
        "-max_muxing_queue_size",
        "2048",
        "-avoid_negative_ts",
        "make_zero",
        "-f",
        "hls",
        "-hls_time",
        "2",
        // EVENT keeps the growing playlist anchored at the requested seek offset.
        "-hls_playlist_type",
        "event",
        "-hls_list_size",
        "0",
        "-hls_flags",
        "temp_file",
        "-hls_segment_filename",
    ])
    .arg(dir.join("segment%06d.ts"))
    .arg(dir.join("index.m3u8"));
    cmd
}
