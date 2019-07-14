use super::error::{empty_result, Error, Result};
use super::opt::Opt;
use ffi;
use libc::c_int;
use std::cell::Cell;
use std::ffi::CStr;
use std::path::Path;
use std::ptr::null_mut;
use std::str::from_utf8_unchecked;

struct AVDictionary {
    inner: Cell<*mut ffi::AVDictionary>,
}

impl Drop for AVDictionary {
    fn drop(&mut self) {
        if self.inner.get() != null_mut() {
            unsafe {
                ffi::av_dict_free(self.inner.as_ptr());
            }
        }
    }
}

impl AVDictionary {
    fn new() -> AVDictionary {
        AVDictionary {
            inner: Cell::new(null_mut()),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        let ret = unsafe {
            let k = CStr::from_bytes_with_nul_unchecked(key.as_bytes());
            let v = CStr::from_bytes_with_nul_unchecked(value.as_bytes());
            ffi::av_dict_set(self.inner.as_ptr(), k.as_ptr(), v.as_ptr(), 0)
        };
        empty_result("av_dict_set(): Failed to set an entry", ret)
    }
}

struct AVFormatContext {
    inner: Cell<*mut ffi::AVFormatContext>,
}

impl Drop for AVFormatContext {
    fn drop(&mut self) {
        if self.inner.get() != null_mut() {
            unsafe {
                ffi::avformat_close_input(self.inner.as_ptr());
            }
        }
    }
}

impl AVFormatContext {
    fn from<P: AsRef<Path>>(p: P) -> Result<AVFormatContext> {
        let path = p.as_ref();
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::new(format!("invalid source: {}", path.display())))?;
        let ctx = AVFormatContext {
            inner: Cell::new(null_mut()),
        };
        let opts = AVDictionary::new();
        opts.set("scan_all_pmts", "1")?;
        let ret = unsafe {
            let source = CStr::from_bytes_with_nul_unchecked(path_str.as_bytes());
            ffi::avformat_open_input(
                ctx.inner.as_ptr(),
                source.as_ptr(),
                null_mut(),
                opts.inner.as_ptr(),
            )
        };
        empty_result("avformat_open_input(): Failed to open file", ret)?;
        Ok(ctx)
    }

    fn check_format(&self) -> Result<()> {
        let name = unsafe {
            let ctx = &*self.inner.get();
            let iformat = &*ctx.iformat;
            from_utf8_unchecked(CStr::from_ptr(iformat.name).to_bytes())
        };
        if name == "mpegts" {
            Ok(())
        } else {
            Err(Error::new(format!(
                "check_format(): Unexpected format - {}",
                name
            )))
        }
    }

    fn find_stream_info(&self) -> Result<()> {
        let ret = unsafe { ffi::avformat_find_stream_info(self.inner.get(), null_mut()) };
        empty_result("find_stream_info(): Stream info not found", ret)
    }

    fn streams(&self) -> impl Iterator<Item = AVStream> {
        let streams = unsafe {
            let ctx = &*self.inner.get();
            std::slice::from_raw_parts(ctx.streams, ctx.nb_streams as usize)
        };
        streams.iter().map(|&stream| AVStream {
            inner: Cell::new(stream),
        })
    }

    fn find_streams(
        &self,
        video_id: Option<u16>,
        audio_id: Option<u16>,
    ) -> Result<(AVStream, AVStream)> {
        let mut video_stream: Option<AVStream> = None;
        let mut audio_stream: Option<AVStream> = None;

        for stream in self.streams() {
            match stream.media_type() {
                ffi::AVMediaType::AVMEDIA_TYPE_VIDEO => {
                    if video_stream.is_none()
                        && (video_id.is_none() || video_id.unwrap() as i32 == stream.id())
                    {
                        stream.discard_default();
                        video_stream = Some(stream);
                    } else {
                        stream.discard_all();
                    }
                }
                ffi::AVMediaType::AVMEDIA_TYPE_AUDIO => {
                    if audio_stream.is_none()
                        && (audio_id.is_none() || audio_id.unwrap() as i32 == stream.id())
                    {
                        stream.discard_default();
                        audio_stream = Some(stream);
                    } else {
                        stream.discard_all();
                    }
                }
                _ => stream.discard_all(),
            }
        }

        let err = |stream: &str, pid: Option<u16>| {
            Err(Error::new(format!(
                "find_stream(): {} stream not found{}",
                stream,
                if let Some(id) = pid {
                    format!(" - PID 0x{:>4x}", id)
                } else {
                    String::new()
                }
            )))
        };
        if video_stream.is_none() {
            return err("Video", video_id);
        }
        if audio_stream.is_none() {
            return err("Audio", audio_id);
        }
        Ok((video_stream.unwrap(), audio_stream.unwrap()))
    }
}

struct AVStream {
    inner: Cell<*mut ffi::AVStream>,
}

impl AVStream {
    fn id(&self) -> c_int {
        unsafe { (*self.inner.get()).id }
    }

    #[inline]
    unsafe fn codecpar_ptr(&self) -> *mut ffi::AVCodecParameters {
        (*self.inner.get()).codecpar
    }

    #[inline]
    fn media_type(&self) -> ffi::AVMediaType {
        unsafe { (*self.codecpar_ptr()).codec_type }
    }

    fn set_discard(&self, discard: ffi::AVDiscard) {
        unsafe {
            let mut stream = &mut *self.inner.get();
            stream.discard = discard;
        }
    }

    fn discard_default(&self) {
        self.set_discard(ffi::AVDiscard::AVDISCARD_DEFAULT);
    }

    fn discard_all(&self) {
        self.set_discard(ffi::AVDiscard::AVDISCARD_ALL);
    }

    fn codec(&self) -> Result<AVCodec> {
        unsafe {
            let params = &*self.codecpar_ptr();
            let codec_ptr = ffi::avcodec_find_decoder(params.codec_id);
            if codec_ptr == null_mut() {
                Err(Error::new(format!(
                    "avcodec_find_decoder(): Codec not found - Codec ID ({:?})",
                    params.codec_id
                )))
            } else {
                Ok(AVCodec { inner: codec_ptr })
            }
        }
    }

    #[inline]
    fn index(&self) -> c_int {
        unsafe { (*self.inner.get()).index }
    }
}

struct AVCodec {
    inner: *const ffi::AVCodec,
}

struct AVCodecContext {
    inner: Cell<*mut ffi::AVCodecContext>,
}

impl Drop for AVCodecContext {
    fn drop(&mut self) {
        if self.inner.get() != null_mut() {
            unsafe {
                ffi::avcodec_free_context(self.inner.as_ptr());
            }
        }
    }
}

impl AVCodecContext {
    fn new(stream: &AVStream) -> Result<AVCodecContext> {
        let codec = stream.codec()?;
        let ctx_ptr = unsafe { ffi::avcodec_alloc_context3(codec.inner) };
        if ctx_ptr == null_mut() {
            return Err(Error::new(
                "avcodec_alloc_context3(): Failed to allocate AVCodecContext",
            ));
        }
        let ctx = AVCodecContext {
            inner: Cell::new(ctx_ptr),
        };
        unsafe {
            let mut ret =
                ffi::avcodec_parameters_to_context(ctx.inner.get(), stream.codecpar_ptr());
            empty_result("avcodec_parameters_to_context(): Failed", ret)?;
            ret = ffi::avcodec_open2(ctx.inner.get(), codec.inner, null_mut());
            empty_result("avcodec_open2(): Failed", ret)?;
        }
        Ok(ctx)
    }

    fn is_valid(&self, packet: &AVPacket) -> bool {
        unsafe { ffi::avcodec_send_packet(self.inner.get(), &packet.inner) == 0 }
    }
}

struct AVPacket {
    inner: ffi::AVPacket,
}

impl Drop for AVPacket {
    fn drop(&mut self) {
        unsafe {
            ffi::av_packet_unref(&mut self.inner);
        }
    }
}

impl AVPacket {
    fn new() -> AVPacket {
        unsafe {
            let mut packet = AVPacket {
                inner: std::mem::zeroed(),
            };
            ffi::av_init_packet(&mut packet.inner);
            packet
        }
    }

    #[inline]
    fn is_keyframe(&self, index: c_int) -> bool {
        self.inner.stream_index == index
            && (self.inner.flags & ffi::AV_PKT_FLAG_KEY) == ffi::AV_PKT_FLAG_KEY
    }

    #[inline]
    fn pts(&self) -> i64 {
        self.inner.pts
    }
    fn check_pts(&self) -> Result<()> {
        if (self.pts() & 0xfffffffe00000000u64 as i64) == 0 {
            Ok(())
        } else {
            Err(Error::new("check_pts(): Detected invalid PTS"))
        }
    }
}

fn fix_overflow(pts1: &mut i64, pts2: &i64) -> bool {
    if (*pts1 & 0x1_8000_0000i64) == 0 && (*pts2 & 0x1_8000_0000i64) == 0x1_8000_0000i64 {
        *pts1 = *pts1 | 0x2_0000_0000;
        true
    } else {
        false
    }
}

pub fn get_delay(opt: &Opt) -> Result<i64> {
    let fmt_ctx = AVFormatContext::from(opt.source())?;
    fmt_ctx.check_format()?;
    fmt_ctx.find_stream_info()?;
    let (video, audio) = fmt_ctx.find_streams(opt.video_id(), opt.audio_id())?;
    let codec_ctx_opt = if opt.drop_broken_audio() {
        Some(AVCodecContext::new(&audio)?)
    } else {
        None
    };
    let mut packet = AVPacket::new();
    let mut video_pts_opt = None;
    let mut audio_pts_opt = None;

    while unsafe { ffi::av_read_frame(fmt_ctx.inner.get(), &mut packet.inner) == 0 } {
        if video_pts_opt.is_none() && packet.is_keyframe(video.index()) {
            packet.check_pts()?;
            video_pts_opt = Some(packet.pts());
        } else if audio_pts_opt.is_none() && packet.is_keyframe(audio.index()) {
            packet.check_pts()?;
            if let Some(ref codec_ctx) = codec_ctx_opt {
                if !codec_ctx.is_valid(&packet) {
                    continue;
                }
            }
            audio_pts_opt = Some(packet.pts());
        }
        if video_pts_opt.is_some() && audio_pts_opt.is_some() {
            break;
        }
    }
    if video_pts_opt.is_none() {
        return Err(Error::new("get_delay(): PTS not found in video stream"));
    }
    if audio_pts_opt.is_none() {
        return Err(Error::new("get_delay(): PTS not found in audio stream"));
    }
    let mut video_pts = video_pts_opt.unwrap();
    let mut audio_pts = audio_pts_opt.unwrap();

    if !fix_overflow(&mut video_pts, &audio_pts) {
        fix_overflow(&mut audio_pts, &video_pts);
    }

    Ok(audio_pts - video_pts)
}

pub fn init_ffmpeg() {
    unsafe {
        ffi::av_log_set_level(ffi::AV_LOG_PANIC);
        ffi::av_register_all();
        ffi::avcodec_register_all();
    }
}
