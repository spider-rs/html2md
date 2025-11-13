extern crate spectral;

#[cfg(feature = "scraper")]
use indoc::indoc;
#[cfg(feature = "scraper")]
use spectral::prelude::*;
#[cfg(feature = "scraper")]
use std::collections::HashMap;

use std::fs::File;
use std::io::prelude::*;
use url::Url;

#[test]
#[ignore]
#[cfg(feature = "scraper")]
fn test_marcfs() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/marcfs-readme.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::parse_html(&html, false);
    assert!(!result.is_empty());
}

#[test]
#[cfg(feature = "scraper")]
fn test_real_world_wiki() -> Result<(), Box<dyn std::error::Error>> {
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::{self, Read};
    use std::path::Path;

    let paths = fs::read_dir("../test-samples/wiki")?;

    fn run_parse(path: &Path) -> Result<(), Box<dyn Error>> {
        let mut html = String::new();
        let mut html_file = File::open(path)?;
        html_file.read_to_string(&mut html)?;

        let result = html2md::parse_html(&html, false);

        if result.is_empty() {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Result is empty",
            )))
        } else {
            Ok(())
        }
    }

    for entry in paths {
        let path = entry?.path();

        if path.is_file() {
            match run_parse(&path) {
                Ok(_) => assert!(true),
                Err(_e) => assert!(false),
            }
        }
    }

    Ok(())
}

#[test]
fn test_real_world_wiki_rewriter() -> Result<(), Box<dyn std::error::Error>> {
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::{self, Read};
    use std::path::Path;

    let paths = fs::read_dir("../test-samples/wiki")?;

    fn run_parse(path: &Path) -> Result<(), Box<dyn Error>> {
        let mut html = String::new();
        let mut html_file = File::open(path)?;
        html_file.read_to_string(&mut html)?;

        let result = html2md::rewrite_html(&html, false);

        if result.is_empty() {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Result is empty",
            )))
        } else {
            Ok(())
        }
    }

    for entry in paths {
        let path = entry?.path();

        if path.is_file() {
            match run_parse(&path) {
                Ok(_) => assert!(true),
                Err(_e) => assert!(false),
            }
        }
    }

    Ok(())
}

#[tokio::test]
#[cfg(all(feature = "stream", feature = "rewriter"))]
async fn test_real_world_wiki_async() -> Result<(), Box<dyn std::error::Error>> {
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::{self, Read};
    use std::path::Path;

    let paths = fs::read_dir("../test-samples/wiki")?;

    async fn run_parse(path: &Path) -> Result<(), Box<dyn Error>> {
        let mut html = String::new();
        let mut html_file = File::open(path)?;
        html_file.read_to_string(&mut html)?;

        let result = html2md::rewrite_html_streaming(&html, false).await;

        if result.is_empty() {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Result is empty",
            )))
        } else {
            Ok(())
        }
    }

    for entry in paths {
        let path = entry?.path();

        if path.is_file() {
            match run_parse(&path).await {
                Ok(_) => assert!(true),
                Err(_e) => assert!(false),
            }
        }
    }

    Ok(())
}

#[test]
#[ignore]
#[cfg(feature = "scraper")]
fn test_real_world_ja() {
    let mut html = String::new();
    let mut html_file: File = File::open("../test-samples/real-world-ja-1.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::parse_html(&html, false);
    assert!(!result.is_empty());
}

#[test]
#[ignore]
#[cfg(feature = "scraper")]
fn test_cheatsheet() {
    let mut html = String::new();
    let mut md = String::new();
    let mut html_file = File::open("../test-samples/markdown-cheatsheet.html").unwrap();
    let mut md_file = File::open("../test-samples/markdown-cheatsheet.md").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    md_file
        .read_to_string(&mut md)
        .expect("File must be readable");
    let md_parsed = html2md::parse_html(&html, false);
    assert!(!md_parsed.is_empty());
}

/// newlines after list shouldn't be converted into text of the last list element
#[test]
#[cfg(feature = "scraper")]
fn test_list_newlines() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-list-newlines.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::parse_html(&html, false);
    assert_that(&result).is_equal_to("xx, xx xxxxx x xxxxxx xxxxxxxx xxxxx xxxxxxxxx xxxx xx xxxx xxxx xxxxxxxx.\nxxxx, xxx xx xxxxx xx xxxxxxxxxxx xxxx.\nxxxxxxxxxxx:\n* xxxxxxx x xxxxxxxxx (xxxxx)\n* xxxxxxx xx xxxxxx xxxxxxx, xxxxxxxxxx xxxxxxxxxx xxxx\n* xxxxxxxxx xx xxxxx, xx xxxxxx xx xxxxxxxxxxx\n* xxxxxxx xxxxxx xxxxxxxxx x xxxxxxxxxx, xxxxxxx xxxxxx x xxxxxxx, x xxxxxx.\n* xx xx, xxxxxx xx xxxxxxxx, xx-xxxx xxx x xxxxxxx xxx xxx, xxxxxxx xx xxxx. xxxxxxxxx xx x.\nxxxxx:\n1. xxxxxxxxx xxxxxxxxxx - xxxxx -\\_- !\n2. xxxxxx Mother of Learning - xxxx, xxxxxxx, xxxxxxxxxxxx\n3. xxxxxx xxxxxxx xxxxxxx, xxxxxxxx \"xxx xxxxx\". xxxxx xxxxx xxxx, xx x xxxxx xxxxxxx.\n4. xxxxxxxx! xxxx xxx xxxxxxxxx xxxx xxx, xx x xxxxxxxxx.\n5. xxxx xxxxxx - xxxxxx xxxxxxxx xxx x 15-17, xxxxxx xxxxxxxxxxxxx xx xxxxxxxx xxx xxxxxxx xxxxxx.\nxxx xxxx, xxxxx x xxxxxxxxx xx xxxxxxxxxx xxxxxx. xxxxxxxxx spelling puns, xxxxxxx, x xxxxxxxxx, xxxxxxxx xxx xxxxxxxx, xxxxxx xxxxxxxxxx xxxxxx.\nxxx xxxxxxx. xxx xxx xxxxxxxx xxxxxx - x x xxxxxxxxxxx xxxxx xxxx xxxxxxxxxx xxx xxxxx, x xxxxxx xxx xxxxxxxx xxxxxxxxxx xxx xxxxx. xx xxxxxx xxxxxxxx:\n* xxx xxxxx x xxx-xxxx xxxxxxxxx. xxxxxx xxx xxxx xxxxxxxx. x xx x xx xxxxxxxx, xx x xxxxxxx xxxxxx xxxxxx xx xxxxxxxxx. xxxxxxxxxx xxxx xxxxx xxxxxx xxxxxxxxx xxxxxxx xx xxxx.\n* xxxxxx xxxx Kotlin, x xxxxxxx. xxxxxxxxxx, xxxxxxxxxx xxx xxxxx xx xxx x xxxxxxxx\n* xxx xxxxx xxxxxxxxxx Rust, xxx xxx x xx xxx xxxx xxxxxxxxx xxxxxxxxxxxxxx xxxx xxx xxxxx, xxxxxxxx xxxxxxxxxxxxxx HTML x Markdown\n* xxx xxxx xxxxxx xxx xxxxxxxx xxxxxx. xx xxxx xxx - xxxxxxxxxxxxx xxxxxxxxxxx xxxxxx x xxxxxxxxx xxxxx x xxxxxxx.\n* xxxxxxxxx xxxx xxxxxxxx xxxxxxx xx FUSE 3.0. xxxxx xxxxxxx xxxxxxx xxx xxxxxxxxxxx.\n* x xxxxxxxx xxxx xxxxxxxx DevOps-xxxxxxx x xxxxx xxxxxxx. xxxxxxxxx, xxx xx xxxxx xxxxxx. x, xx, xxx xxx xxx xxxxxxxxx?\nxxxxx xx xxx:\n\\- xxxxxxxx xxxxxxxx\n\\- xxxxxxx xxxxxxxxx, xxxxxxx xxxxx xxxxx xxxxxxxx\n\\- xxxxxxxxxx xxxx Machine Learning, xxxx xxxxxx xxx xxxxxxxx OpenCL.".to_string());
}

#[test]
#[cfg(feature = "scraper")]
fn test_lists_from_text() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-lists-from-text.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::parse_html(&html, false);
    assert_that(&result).contains("\\- x xxxx xxxxx xx xxxxxxxxxx");
    assert_that(&result).contains("\\- x xxxx xxxxxxxx xxxxxxxxx xxxxxx xxx x xxxxxxxx xxxx");
    assert_that(&result).contains("\\- xxxx xxxxxxxx");
}

#[test]
#[cfg(feature = "scraper")]
fn test_strong_inside_link() {
    let mut html = String::new();
    let mut html_file =
        File::open("../test-samples/dybr-bug-with-strong-inside-link.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::parse_html(&html, false);
    assert_that(&result).contains("[**Just God**](http://fanfics.me/ficXXXXXXX)");
}

#[test]
#[cfg(feature = "scraper")]
fn test_tables_with_newlines() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-tables-masked.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::parse_html(&html, false);
    let m = indoc! { "[![Embedded YouTube video](https://img.youtube.com/vi/ZZZZZZZZZ/0.jpg)](https://www.youtube.com/watch?v=ZZZZZZZZZ)\n|Maybe I'm foolish, maybe I'm blind\nThinking I can see through this and see what's behind\nGot no way to prove it so maybe I'm blind\nBut I'm only human after all,\nI'm only human after all\nDon't put your blame on me|xxxxx xxxx, x xxxxxx, xxxxx xxxx —xxxxxx\nxxx xxxxx, xxx xxxx xxxxxx xxxxxx xxx, x xxxxxx xxx xxx xx xxx\nxxxx x xxxx xx xxxx xxxxxxx xxxxxxxxxxxxx, xxx xxx xxxxxxxx, x xxxxxx.\nxx x xxxxx xxxx xxxxxxx, x xxxxx-xx xxxxxx,\nx xxxxx xxxx xxxxxxx, x xxxxx xxxxxx.\nxx xxxx xxxx|\n|||\n[xxxxxx xxxxx xxxxx x xxxxxxx](/)\nx xxxx xxxxxxxxx xxxxxxx xxxxxxxxxxx xx xxxx xxxxx. x xxxxx xxxxxxx, xxxx xxxxx xxxxxxx xx xxxxxxxxxx xxxxxx. xxx xxxxxxxx, xxx xxxxxxxxx xxxxxxxxxxxxxx xx xxxxx —xxxxxxxxxx xxxxxxxxxx x xxxxx xxxxxxxxxxxxx xxxxxxxxx. x xxx xxxxxxxxxxxx*xxxx*, xxxxxx xxxx, xxxxxxxxxx xxxxx xxxxxxxx, xxxxxxxxxx x xxxxxxxxx. xx xxxxxx xxxxx xxxxxxxxxxxxxxxxx —x xxxxxx xxx xxxx.\nxxxxx xxxxxxxxxx xxxxx x xxxx xxxxxxxxxx xxxxx. xxxxx. x xxxxx: «x xxxxxx xxxxxxx, x xxxxx xxx xxxx, xx xxxxxxxx xxxxxx», —xxx xxxxx xxxxxxxx. xxxxxx xxx x xxxx xxxx xxxxxxxx xxxxxxxx xxxxxxx xxxx xxxxxxxxxxx xxxxxxxxxx, xxxxxxx xxxxxx xxxxxx xxx xxxxx, xxxxxxxxxxx x x xxxxxxx xxxxxxxxx.\nxx x xxxxx xxxx xxxxxxx. xxxxxx xxxxx? xxxxxxxxxxx x xxxxxxxxx xxxxxx.\nx xxxxx x xxxxxxxxxx x xxxxx... x xxxxxx xxxx xxxxxx xxxxxxx xxxxxxxx. xx xxxx, x xxxxxx xxx-xx xxxxxxxxx xx xxxxxxx, xxx xxxxxx xxxxxx, xxx xxx xxxxx, xxxxx xxxxxxxx xx xxxx... x xxxxxx xxxxxxx xx xxxx xxxxx, xxx, xxxxx xxxx xxxxxxxxxx, x xxxxx xxxxxxxxx xx xxxxx. x xxx-xx xxx xxxxx xxxxxxx xxxxxxxxxxxxx.\nxxxxxx xx... xx xxx xx xxxxxxxxxxxxx xxxxxx xxxxxxxxxxxxx x xxxxxxxxxx xxxxx, xxxxx xxx xxxx xxxxxxxxx, x xxxxx xxx xxxxxxxxx, xxx xxxxxxx xxx, xxx xxxx xxxxxxx xxxxxx, x xx xxx, xxx xxxx xxxxxxxx." };

    assert_that!(result).contains(m);
    // let result = html2md::rewrite_html(&html, false);
    // assert_that!(result).contains(m);
}

#[test]
#[cfg(feature = "scraper")]
fn test_tables_crash2() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-tables-2-masked.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let table_with_vertical_header = html2md::parse_html(&html, false);
    let m = indoc! {"xxxxx xxxxxxxxxx xxxxxxx x xxxxx))~~xxxxxxxx xxxxxxxx~~\n## At a Glance\n|Current Conditions:|Open all year. No reservations. No services.|\n|||\n| Reservations: | No reservations. |\n| Fees | No fee. |\n| Water: | No water. |"};

    assert_that!(table_with_vertical_header).contains(m);

    let table_with_vertical_header = html2md::rewrite_html(&html, false);

    let m = indoc! { "xxxxx xxxxxxxxxx xxxxxxx x xxxxx))~~xxxxxxxx xxxxxxxx~~\n## At a Glance\n|Current Conditions:|Open all year. No reservations. No services.|\nReservations:|No reservations.|\nFees|No fee.|\nWater:|No water.|"};

    assert_that!(table_with_vertical_header).contains(m);
}

#[test]
#[cfg(feature = "scraper")]
fn test_html_from_text() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/real-world-1.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");

    let mut tag_factory: HashMap<String, Box<dyn html2md::scraper::TagHandlerFactory>> =
        HashMap::new();
    let tag = Box::new(html2md::scraper::ignore::IgnoreTagFactory {});

    tag_factory.insert(String::from("script"), tag.clone());
    tag_factory.insert(String::from("style"), tag.clone());
    tag_factory.insert(String::from("noscript"), tag.clone());

    tag_factory.insert(String::from("iframe"), tag);

    let result = html2md::parse_html_custom_with_url(
        &html,
        &tag_factory,
        false,
        &Some(Url::parse("https://spider.cloud").unwrap()),
    );

    assert!(!result.is_empty());
}

#[test]
#[cfg(all(feature = "stream", feature = "rewriter"))]
fn test_html_from_text_rewrite() {
    let mut html = Box::new(String::new());
    let mut html_file = File::open("../test-samples/real-world-1.html").unwrap();

    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");

    let result = html2md::rewrite_html_custom_with_url(
        &html,
        &None,
        false,
        &Some(Url::parse("https://spider.cloud").unwrap()),
    );

    assert!(!result.is_empty());
}

const SPIDER_RESULT_MD: &str = r#"To help you get started with Spider, we’ll give you $200 in credits when you spend $100.[Terms apply](https://spider.cloud/promotion-spider-credits)
# The Web Crawler for AI Agents and LLMs
Spider offers the finest data collecting solution. Engineered for speed and scalability, it
allows you to elevate your AI projects.
[Get Started](https://spider.cloud/credits/new)View Preview
* Basic
* Streaming
Example request
Python
JSONL
Copy
```
`import requests, os, json
headers = {
&#x27;&#x27;Authorization &#x27;&#x27;: f &#x27;&#x27;Bearer {os.getenv(&quot;&quot;SPIDER\_API\_KEY &quot;&quot;)}&#x27;&#x27;,
&#x27;&#x27;Content-Type &#x27;&#x27;: &#x27;&#x27;application/jsonl &#x27;&#x27;,
}
json\_data = {&quot;&quot;limit &quot;&quot;:50,&quot;&quot;metadata &quot;&quot;:True,&quot;&quot;url &quot;&quot;:&quot;&quot;https://spider.cloud &quot;&quot;}
response = requests.post(&#x27;&#x27;https://api.spider.cloud/crawl &#x27;&#x27;, headers=headers, json=json\_data, stream=True)
with response as r:
r.raise\_for\_status()
for chunk in r.iter\_lines(
chunk\_size=None, decode\_unicode=True
):
data = json.loads(chunk)
print(data)`
```
[Free Trial](https://spider.cloud/credits/new?free-trial=1)
Example Response
## Built with the need for**Speed**
Experience the power of**Spider**, built fully in**Rust**for
next-generation scalability.
### 2.4secs
To crawl over 20,000 pages
### 500-1000x
Faster than alternatives
### 500x
Cheaper than traditional scraping services
Benchmarks displaying performance between Spider API request modes.
Spider API Request Modes &middot;Benchmarked tailwindcss.com &middot;06/16/2024
[See framework benchmarks](https://github.com/spider-rs/spider/blob/main/benches/BENCHMARKS.md)
### Seamless Integrations
Seamlessly integrate Spider with a wide range of platforms, ensuring data curation
perfectly aligned with your requirements. Compatible with all major AI tools.
[LangChain integration](https://python.langchain.com/docs/integrations/document_loaders/spider)[LlamaIndex integrationLlama Index Logo](https://docs.llamaindex.ai/en/stable/examples/data_connectors/WebPageDemo/#using-spider-reader)[CrewAI integrationCrewAI Logo](https://docs.crewai.com/tools/SpiderTool/)[FlowWiseAI integrationFlowiseAI LogoFlowiseAI](https://docs.flowiseai.com/integrations/langchain/document-loaders/spider-web-scraper-crawler)[Composio integrationComposio Logo](https://docs.composio.dev/introduction/foundations/components/list_local_tools#spider-crawler)[PhiData integrationPhiData Logo](https://docs.phidata.com/tools/spider)
### Concurrent Streaming
Save time and money without having to worry about bandwidth concerns by effectively
streaming all the results concurrently. The latency cost that is saved becomes drastic as
you crawl more websites.
### Warp Speed
Powered by the cutting-edge[Spider](https://github.com/spider-rs/spider)open-source project, our robust Rust engine scales effortlessly to handle extreme
workloads. We ensure continuous maintenance and improvement for top-tier performance.
## Kickstart Your Data Collecting Projects Today
Jumpstart web crawling with full elastic scaling concurrency, optimal formats, and AI scraping.
### Performance Tuned
Spider is written in Rust and runs in full concurrency to achieve crawling thousands of
pages in secs.
### Multiple response formats
Get clean and formatted markdown, HTML, or text content for fine-tuning or training AI
models.
### Caching
Further boost speed by caching repeated web page crawls to minimize expenses while
building.
### Smart Mode
Spider dynamically switches to Headless Chrome when it needs to quick.
Beta
### Scrape with AI
Do custom browser scripting and data extraction using the latest AI models with no cost
step caching.
### The crawler for LLMs
Don't let crawling and scraping be the highest latency in your LLM & AI agent stack.
### Scrape with no headaches
* Auto Proxy rotations
* Agent headers
* Anti-bot detections
* Headless chrome
* Markdown responses
### The Fastest Web Crawler
* Powered by[spider-rs](https://github.com/spider-rs/spider)
* 100,000 pages/seconds
* Unlimited concurrency
* Simple API
* 50,000 RPM
### Do more with AI
* Browser scripting
* Advanced extraction
* Data pipelines
* Ideal for LLMs and AI Agents
* Accurate labeling
## Achieve more with these new API features
Our API is set to stream so you can act in realtime.
![A user interface with a search bar containing the text &#34;Latest sports news,&#34; a green &#34;Submit&#34; button, and two icon buttons to display searching and extracting with the service.](/img/search_feature.webp)
### Search
Get access to search engine results from anywhere and easily crawl and transform pages to
LLM-ready markdown.
[Explore SearchRight Arrow](https://spider.cloud/docs/api#search)
![A user interface segment showing three icons representing different stages of data transformation.](/img/transform_feature_example.webp)
### Transform
Convert raw HTML into markdown easily by using this API. Transform thousands of html pages
in seconds.
[Explore TransformRight Arrow](https://spider.cloud/docs/api#transform)
## Join the community
Backed by a network of early advocates, contributors, and supporters.
[GitHub discussions
Chat Icon
](https://github.com/orgs/spider-rs/discussions)[Discord
Chat Icon
](https://discord.spider.cloud)
[
![iammerrick's avatar](/img/external/iammerrick_twitter.webp)
@iammerrick
Rust based crawler Spider is next level for crawling &amp;scraping sites. So fast.
Their cloud offering is also so easy to use. Good stuff. https://github.com/spider-rs/spider
](https://twitter.com/iammerrick/status/1787873425446572462)
[
![WilliamEspegren's avatar](/img/external/william_twitter.webp)
@WilliamEspegren
Web crawler built in rust, currently the nr1 performance in the world with crazy resource management Aaaaaaand they have a cloud offer, that’s wayyyy cheaper than any competitor
Name a reason for me to use anything else?
github.com/spider-rs/spid…
](https://twitter.com/WilliamEspegren/status/1789419820821184764)
[
![gasa's avatar](/img/external/gaza_twitter.webp)
@gasa
@gasathenaper
is the best crawling tool i have used. I had a complicated project where i needed to paste url and get the website whole website data. Spider does it in an instant
](https://x.com/gasathenaper/status/1810612492596383948)
[
![Ashpreet Bedi's avatar](/img/external/ashpreet_bedi.webp)
@Ashpreet Bedi
@ashpreetbedi
is THE best crawler out there, give it a try
](https://x.com/ashpreetbedi/status/1815512219003572315?s=46&t=37F5QP_8oKqOsNpHSo6VVw)
[
![Troyusrex's avatar](/img/external/troy_twitter.webp)
@Troyusrex
I found a new tool, Spider-rs, which scrapes significantly faster and handles more scenarios than the basic scraper I built did. Our use of Spider-rs and AWS infrastructure reduced the scraping time from four months to under a week.
](https://medium.com/@troyusrex/inside-my-virtual-college-advisor-a-deep-dive-into-rag-ai-and-agent-technology-84731b2928f7#1326)
[
![Dify.AI's avatar](/img/external/difyai.webp)
@Dify.AI
🕷️Spider @spider\_rust
can be used as a built-in tool in #Dify Workflow or as an LLM-callable tool in Agent. It allows fast and affordable web scraping and crawling when your AI applications need real-time web data for context.
](https://x.com/dify_ai/status/1818226971056243089)
## FAQ
Frequently asked questions about Spider.
### What is Spider?
Spider is a leading web crawling tool designed for speed and cost-effectiveness, supporting various data formats including LLM-ready markdown.
### Why is my website not crawling?
Your crawl may fail if it requires JavaScript rendering. Try setting your request to &#x27;chrome &#x27;to solve this issue.
### Can you crawl all pages?
Yes, Spider accurately crawls all necessary content without needing a sitemap.
### What formats can Spider convert web data into?
Spider outputs HTML, raw, text, and various markdown formats. It supports`JSON`,`JSONL`,`CSV`, and`XML`for API responses.
### Is Spider suitable for large scraping projects?
Absolutely, Spider is ideal for large-scale data collection and offers a cost-effective dashboard for data management.
### How can I try Spider?
Purchase credits for our cloud system or test the Open Source Spider engine to explore its capabilities.
### Does it respect robots.txt?
Yes, compliance with robots.txt is default, but you can disable this if necessary.
### Unable to get dynamic content?
If you are having trouble getting dynamic pages, try setting the request parameter to &quot;&quot;chrome &quot;&quot;or &quot;&quot;smart.&quot;&quot;You may also need to set `disable\_intercept` to allow third-party or external scripts to run.
### Why is my crawl going slow?
If you are experiencing a slow crawl, it is most likely due to the robots.txt file for the website. The robots.txt file may have a crawl delay set, and we respect the delay up to 60 seconds.
### Do you offer a Free Trial?
Yes, you can try out the service before being charged for free at[checkout](https://spider.cloud/credits/new?free-trial=1).
## Comprehensive Data Curation for Everyone
Trusted by leading tech businesses worldwide to deliver accurate and insightful data solutions.
Outer Labs
[Zapier LogoZapier](https://zapier.com/apps/spider/integrations)
Elementus Logo
Super AI Logo
LayerX Logo
Swiss Re
Write Sonic Logo
Alioth Logo
### Next generation data for AI, scale to millions
[Start now](https://spider.cloud/credits/new)
### Company
* [About](https://spider.cloud/about)
* [Privacy](https://spider.cloud/privacy)
* [Terms](https://spider.cloud/eula)
* [FAQ](https://spider.cloud/faq)
### Resources
* [API](https://spider.cloud/docs/api)
* [Docs](https://spider.cloud/docs/overview)
* [Guides](https://spider.cloud/guides)
* [Spider.rs Docs](https://docs.rs/spider/latest/spider/)
### Services
* [Pricing](https://spider.cloud/credits/new)
* [Web Crawling and Scraping](https://spider.cloud/web-crawling-and-scraping)
[All systems normal.](https://spidercloud.statuspage.io/)
[
Github LogoGitHub
](https://github.com/spider-rs/spider)[
Discord LogoDiscord
](https://discord.spider.cloud)[
Twitter LogoTwitter
](https://twitter.com/spider_rust)"#;

const EXAMPLE_RESULT_MD: &str = r###"Example Domain
# Example Domain
This domain is for use in documentation examples without needing permission. Avoid use in operations.
[Learn more](https://iana.org/domains/example)"###;

#[test]
#[ignore]
fn test_real_spider() {
    let mut html = String::new();
    let mut html_file: File = File::open("../test-samples/spider-cloud.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::rewrite_html(&html, false);
    assert!(result == SPIDER_RESULT_MD);
}

#[tokio::test]
#[ignore]
#[cfg(all(feature = "stream", feature = "rewriter"))]
async fn test_real_spider_async() {
    let mut html = String::new();
    let mut html_file: File = File::open("../test-samples/spider-cloud.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::rewrite_html_streaming(&html, false).await;
    assert!(result == SPIDER_RESULT_MD);
}

#[tokio::test]
#[ignore]
#[cfg(all(feature = "stream", feature = "rewriter"))]
async fn test_real_spider_async_basic() {
    let mut html = String::new();
    let mut html_file: File = File::open("../test-samples/example.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = html2md::rewrite_html_streaming(&html, false).await;
    assert!(result == EXAMPLE_RESULT_MD);
}
