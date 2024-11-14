use regex::Regex;

lazy_static::lazy_static! {
    /// Pattern that detects iframes with Youtube embedded videos<br/>
    /// Examples:
    /// * `https://www.youtube.com/embed/zE-dmXZp3nU?wmode=opaque`
    /// * `https://www.youtube-nocookie.com/embed/5yo6exIypkY`
    /// * `https://www.youtube.com/embed/TXm6IXrbQuM`
    pub(crate) static ref YOUTUBE_PATTERN : Regex = Regex::new(r"www\.youtube(?:-nocookie)?\.com/embed/([-\w]+)").expect("valid regex pattern");

    /// Pattern that detects iframes with Instagram embedded photos<br/>
    /// Examples:
    /// * `https://www.instagram.com/p/B1BKr9Wo8YX/embed/`
    /// * `https://www.instagram.com/p/BpKjlo-B4uI/embed/`
    pub(crate) static ref INSTAGRAM_PATTERN: Regex = Regex::new(r"www\.instagram\.com/p/([-\w]+)/embed").expect("valid regex pattern");

    /// Patter that detects iframes with VKontakte embedded videos<br/>
    /// Examples:
    /// * `https://vk.com/video_ext.php?oid=-49423435&id=456245092&hash=e1611aefe899c4f8`
    /// * `https://vk.com/video_ext.php?oid=-76477496&id=456239454&hash=ebfdc2d386617b97`
    pub(crate) static ref VK_PATTERN: Regex = Regex::new(r"vk\.com/video_ext\.php\?oid=(-?\d+)&id=(\d+)&hash=(.*)").expect("valid regex pattern");
}
