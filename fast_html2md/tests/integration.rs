extern crate spectral;

// use html2md::ignore::IgnoreTagFactory;
// use html2md::{parse_html, parse_html_custom, parse_html_custom_with_url};
use html2md::{parse_html, rewrite_html};
use indoc::indoc;
use spectral::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use url::Url;

#[test]
#[ignore]
fn test_marcfs() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/marcfs-readme.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert!(!result.is_empty());
}

#[test]
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

        let result = parse_html(&html, false);

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
#[ignore]
fn test_real_world_ja() {
    let mut html = String::new();
    let mut html_file: File = File::open("../test-samples/real-world-ja-1.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert!(!result.is_empty());
}

#[test]
#[ignore]
fn test_real_spider() {
    let mut html = String::new();
    let mut html_file: File = File::open("../test-samples/spider-cloud.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = rewrite_html(&html, false);
    assert!(result == r#"To help you get started with Spider, we’ll give you $200 in credits when you spend $100.[Terms apply](https://spider.cloud/promotion-spider-credits)\n# The Web Crawler for AI Agents and LLMs\nSpider offers the finest data collecting solution. Engineered for speed and scalability, it\nallows you to elevate your AI projects.\n[Get Started](https://spider.cloud/credits/new)View Preview\n* Basic\n* Streaming\nExample request\nPython\nJSONL\nCopy\n```\n`import requests, os, json\nheaders = {\n&#x27;&#x27;Authorization &#x27;&#x27;: f &#x27;&#x27;Bearer {os.getenv(&quot;&quot;SPIDER\\_API\\_KEY &quot;&quot;)}&#x27;&#x27;,\n&#x27;&#x27;Content-Type &#x27;&#x27;: &#x27;&#x27;application/jsonl &#x27;&#x27;,\n}\njson\\_data = {&quot;&quot;limit &quot;&quot;:50,&quot;&quot;metadata &quot;&quot;:True,&quot;&quot;url &quot;&quot;:&quot;&quot;https://spider.cloud &quot;&quot;}\nresponse = requests.post(&#x27;&#x27;https://api.spider.cloud/crawl &#x27;&#x27;, headers=headers, json=json\\_data, stream=True)\nwith response as r:\nr.raise\\_for\\_status()\nfor chunk in r.iter\\_lines(\nchunk\\_size=None, decode\\_unicode=True\n):\ndata = json.loads(chunk)\nprint(data)`\n```\n[Free Trial](https://spider.cloud/credits/new?free-trial=1)\nExample Response\n## Built with the need for**Speed**\nExperience the power of**Spider**, built fully in**Rust**for\nnext-generation scalability.\n### 2.4secs\nTo crawl over 20,000 pages\n### 500-1000x\nFaster than alternatives\n### 500x\nCheaper than traditional scraping services\nBenchmarks displaying performance between Spider API request modes.\nSpider API Request Modes &middot;Benchmarked tailwindcss.com &middot;06/16/2024\n[See framework benchmarks](https://github.com/spider-rs/spider/blob/main/benches/BENCHMARKS.md)\n### Seamless Integrations\nSeamlessly integrate Spider with a wide range of platforms, ensuring data curation\nperfectly aligned with your requirements. Compatible with all major AI tools.\n[LangChain integration](https://python.langchain.com/docs/integrations/document_loaders/spider)[LlamaIndex integrationLlama Index Logo](https://docs.llamaindex.ai/en/stable/examples/data_connectors/WebPageDemo/#using-spider-reader)[CrewAI integrationCrewAI Logo](https://docs.crewai.com/tools/SpiderTool/)[FlowWiseAI integrationFlowiseAI LogoFlowiseAI](https://docs.flowiseai.com/integrations/langchain/document-loaders/spider-web-scraper-crawler)[Composio integrationComposio Logo](https://docs.composio.dev/introduction/foundations/components/list_local_tools#spider-crawler)[PhiData integrationPhiData Logo](https://docs.phidata.com/tools/spider)\n### Concurrent Streaming\nSave time and money without having to worry about bandwidth concerns by effectively\nstreaming all the results concurrently. The latency cost that is saved becomes drastic as\nyou crawl more websites.\n### Warp Speed\nPowered by the cutting-edge[Spider](https://github.com/spider-rs/spider)open-source project, our robust Rust engine scales effortlessly to handle extreme\nworkloads. We ensure continuous maintenance and improvement for top-tier performance.\n## Kickstart Your Data Collecting Projects Today\nJumpstart web crawling with full elastic scaling concurrency, optimal formats, and AI scraping.\n### Performance Tuned\nSpider is written in Rust and runs in full concurrency to achieve crawling thousands of\npages in secs.\n### Multiple response formats\nGet clean and formatted markdown, HTML, or text content for fine-tuning or training AI\nmodels.\n### Caching\nFurther boost speed by caching repeated web page crawls to minimize expenses while\nbuilding.\n### Smart Mode\nSpider dynamically switches to Headless Chrome when it needs to quick.\nBeta\n### Scrape with AI\nDo custom browser scripting and data extraction using the latest AI models with no cost\nstep caching.\n### The crawler for LLMs\nDon't let crawling and scraping be the highest latency in your LLM & AI agent stack.\n### Scrape with no headaches\n* Auto Proxy rotations\n* Agent headers\n* Anti-bot detections\n* Headless chrome\n* Markdown responses\n### The Fastest Web Crawler\n* Powered by[spider-rs](https://github.com/spider-rs/spider)\n* 100,000 pages/seconds\n* Unlimited concurrency\n* Simple API\n* 50,000 RPM\n### Do more with AI\n* Browser scripting\n* Advanced extraction\n* Data pipelines\n* Ideal for LLMs and AI Agents\n* Accurate labeling\n## Achieve more with these new API features\nOur API is set to stream so you can act in realtime.\n![A user interface with a search bar containing the text &#34;Latest sports news,&#34; a green &#34;Submit&#34; button, and two icon buttons to display searching and extracting with the service.](/img/search_feature.webp)\n### Search\nGet access to search engine results from anywhere and easily crawl and transform pages to\nLLM-ready markdown.\n[Explore SearchRight Arrow](https://spider.cloud/docs/api#search)\n![A user interface segment showing three icons representing different stages of data transformation.](/img/transform_feature_example.webp)\n### Transform\nConvert raw HTML into markdown easily by using this API. Transform thousands of html pages\nin seconds.\n[Explore TransformRight Arrow](https://spider.cloud/docs/api#transform)\n## Join the community\nBacked by a network of early advocates, contributors, and supporters.\n[GitHub discussions\nChat Icon\n](https://github.com/orgs/spider-rs/discussions)[Discord\nChat Icon\n](https://discord.spider.cloud)\n[\n![iammerrick's avatar](/img/external/iammerrick_twitter.webp)\n@iammerrick\nRust based crawler Spider is next level for crawling &amp;scraping sites. So fast.\nTheir cloud offering is also so easy to use. Good stuff. https://github.com/spider-rs/spider\n](https://twitter.com/iammerrick/status/1787873425446572462)\n[\n![WilliamEspegren's avatar](/img/external/william_twitter.webp)\n@WilliamEspegren\nWeb crawler built in rust, currently the nr1 performance in the world with crazy resource management Aaaaaaand they have a cloud offer, that’s wayyyy cheaper than any competitor\nName a reason for me to use anything else?\ngithub.com/spider-rs/spid…\n](https://twitter.com/WilliamEspegren/status/1789419820821184764)\n[\n![gasa's avatar](/img/external/gaza_twitter.webp)\n@gasa\n@gasathenaper\nis the best crawling tool i have used. I had a complicated project where i needed to paste url and get the website whole website data. Spider does it in an instant\n](https://x.com/gasathenaper/status/1810612492596383948)\n[\n![Ashpreet Bedi's avatar](/img/external/ashpreet_bedi.webp)\n@Ashpreet Bedi\n@ashpreetbedi\nis THE best crawler out there, give it a try\n](https://x.com/ashpreetbedi/status/1815512219003572315?s=46&t=37F5QP_8oKqOsNpHSo6VVw)\n[\n![Troyusrex's avatar](/img/external/troy_twitter.webp)\n@Troyusrex\nI found a new tool, Spider-rs, which scrapes significantly faster and handles more scenarios than the basic scraper I built did. Our use of Spider-rs and AWS infrastructure reduced the scraping time from four months to under a week.\n](https://medium.com/@troyusrex/inside-my-virtual-college-advisor-a-deep-dive-into-rag-ai-and-agent-technology-84731b2928f7#1326)\n[\n![Dify.AI's avatar](/img/external/difyai.webp)\n@Dify.AI\n🕷\u{fe0f}Spider @spider\\_rust\ncan be used as a built-in tool in #Dify Workflow or as an LLM-callable tool in Agent. It allows fast and affordable web scraping and crawling when your AI applications need real-time web data for context.\n](https://x.com/dify_ai/status/1818226971056243089)\n## FAQ\nFrequently asked questions about Spider.\n### What is Spider?\nSpider is a leading web crawling tool designed for speed and cost-effectiveness, supporting various data formats including LLM-ready markdown.\n### Why is my website not crawling?\nYour crawl may fail if it requires JavaScript rendering. Try setting your request to &#x27;chrome &#x27;to solve this issue.\n### Can you crawl all pages?\nYes, Spider accurately crawls all necessary content without needing a sitemap.\n### What formats can Spider convert web data into?\nSpider outputs HTML, raw, text, and various markdown formats. It supports`JSON`,`JSONL`,`CSV`, and`XML`for API responses.\n### Is Spider suitable for large scraping projects?\nAbsolutely, Spider is ideal for large-scale data collection and offers a cost-effective dashboard for data management.\n### How can I try Spider?\nPurchase credits for our cloud system or test the Open Source Spider engine to explore its capabilities.\n### Does it respect robots.txt?\nYes, compliance with robots.txt is default, but you can disable this if necessary.\n### Unable to get dynamic content?\nIf you are having trouble getting dynamic pages, try setting the request parameter to &quot;&quot;chrome &quot;&quot;or &quot;&quot;smart.&quot;&quot;You may also need to set `disable\\_intercept` to allow third-party or external scripts to run.\n### Why is my crawl going slow?\nIf you are experiencing a slow crawl, it is most likely due to the robots.txt file for the website. The robots.txt file may have a crawl delay set, and we respect the delay up to 60 seconds.\n### Do you offer a Free Trial?\nYes, you can try out the service before being charged for free at[checkout](https://spider.cloud/credits/new?free-trial=1).\n## Comprehensive Data Curation for Everyone\nTrusted by leading tech businesses worldwide to deliver accurate and insightful data solutions.\nOuter Labs\n[Zapier LogoZapier](https://zapier.com/apps/spider/integrations)\nElementus Logo\nSuper AI Logo\nLayerX Logo\nSwiss Re\nWrite Sonic Logo\nAlioth Logo\n### Next generation data for AI, scale to millions\n[Start now](https://spider.cloud/credits/new)\n### Company\n* [About](https://spider.cloud/about)\n* [Privacy](https://spider.cloud/privacy)\n* [Terms](https://spider.cloud/eula)\n* [FAQ](https://spider.cloud/faq)\n### Resources\n* [API](https://spider.cloud/docs/api)\n* [Docs](https://spider.cloud/docs/overview)\n* [Guides](https://spider.cloud/guides)\n* [Spider.rs Docs](https://docs.rs/spider/latest/spider/)\n### Services\n* [Pricing](https://spider.cloud/credits/new)\n* [Web Crawling and Scraping](https://spider.cloud/web-crawling-and-scraping)\n[All systems normal.](https://spidercloud.statuspage.io/)\n[\nGithub LogoGitHub\n](https://github.com/spider-rs/spider)[\nDiscord LogoDiscord\n](https://discord.spider.cloud)[\nTwitter LogoTwitter\n](https://twitter.com/spider_rust)"#);
}

#[test]
#[ignore]
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
    let md_parsed = parse_html(&html, false);
    assert!(!md_parsed.is_empty());
}

/// newlines after list shouldn't be converted into text of the last list element
#[test]
fn test_list_newlines() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-list-newlines.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert_that(&result).is_equal_to("xx, xx xxxxx x xxxxxx xxxxxxxx xxxxx xxxxxxxxx xxxx xx xxxx xxxx xxxxxxxx.\nxxxx, xxx xx xxxxx xx xxxxxxxxxxx xxxx.\nxxxxxxxxxxx:\n* xxxxxxx x xxxxxxxxx (xxxxx)\n* xxxxxxx xx xxxxxx xxxxxxx, xxxxxxxxxx xxxxxxxxxx xxxx\n* xxxxxxxxx xx xxxxx, xx xxxxxx xx xxxxxxxxxxx\n* xxxxxxx xxxxxx xxxxxxxxx x xxxxxxxxxx, xxxxxxx xxxxxx x xxxxxxx, x xxxxxx.\n* xx xx, xxxxxx xx xxxxxxxx, xx-xxxx xxx x xxxxxxx xxx xxx, xxxxxxx xx xxxx. xxxxxxxxx xx x.\nxxxxx:\n1. xxxxxxxxx xxxxxxxxxx - xxxxx -\\_- !\n2. xxxxxx Mother of Learning - xxxx, xxxxxxx, xxxxxxxxxxxx\n3. xxxxxx xxxxxxx xxxxxxx, xxxxxxxx \"xxx xxxxx\". xxxxx xxxxx xxxx, xx x xxxxx xxxxxxx.\n4. xxxxxxxx! xxxx xxx xxxxxxxxx xxxx xxx, xx x xxxxxxxxx.\n5. xxxx xxxxxx - xxxxxx xxxxxxxx xxx x 15-17, xxxxxx xxxxxxxxxxxxx xx xxxxxxxx xxx xxxxxxx xxxxxx.\nxxx xxxx, xxxxx x xxxxxxxxx xx xxxxxxxxxx xxxxxx. xxxxxxxxx spelling puns, xxxxxxx, x xxxxxxxxx, xxxxxxxx xxx xxxxxxxx, xxxxxx xxxxxxxxxx xxxxxx.\nxxx xxxxxxx. xxx xxx xxxxxxxx xxxxxx - x x xxxxxxxxxxx xxxxx xxxx xxxxxxxxxx xxx xxxxx, x xxxxxx xxx xxxxxxxx xxxxxxxxxx xxx xxxxx. xx xxxxxx xxxxxxxx:\n* xxx xxxxx x xxx-xxxx xxxxxxxxx. xxxxxx xxx xxxx xxxxxxxx. x xx x xx xxxxxxxx, xx x xxxxxxx xxxxxx xxxxxx xx xxxxxxxxx. xxxxxxxxxx xxxx xxxxx xxxxxx xxxxxxxxx xxxxxxx xx xxxx.\n* xxxxxx xxxx Kotlin, x xxxxxxx. xxxxxxxxxx, xxxxxxxxxx xxx xxxxx xx xxx x xxxxxxxx\n* xxx xxxxx xxxxxxxxxx Rust, xxx xxx x xx xxx xxxx xxxxxxxxx xxxxxxxxxxxxxx xxxx xxx xxxxx, xxxxxxxx xxxxxxxxxxxxxx HTML x Markdown\n* xxx xxxx xxxxxx xxx xxxxxxxx xxxxxx. xx xxxx xxx - xxxxxxxxxxxxx xxxxxxxxxxx xxxxxx x xxxxxxxxx xxxxx x xxxxxxx.\n* xxxxxxxxx xxxx xxxxxxxx xxxxxxx xx FUSE 3.0. xxxxx xxxxxxx xxxxxxx xxx xxxxxxxxxxx.\n* x xxxxxxxx xxxx xxxxxxxx DevOps-xxxxxxx x xxxxx xxxxxxx. xxxxxxxxx, xxx xx xxxxx xxxxxx. x, xx, xxx xxx xxx xxxxxxxxx?\nxxxxx xx xxx:\n\\- xxxxxxxx xxxxxxxx\n\\- xxxxxxx xxxxxxxxx, xxxxxxx xxxxx xxxxx xxxxxxxx\n\\- xxxxxxxxxx xxxx Machine Learning, xxxx xxxxxx xxx xxxxxxxx OpenCL.".to_string());
}

#[test]
fn test_lists_from_text() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-lists-from-text.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert_that(&result).contains("\\- x xxxx xxxxx xx xxxxxxxxxx");
    assert_that(&result).contains("\\- x xxxx xxxxxxxx xxxxxxxxx xxxxxx xxx x xxxxxxxx xxxx");
    assert_that(&result).contains("\\- xxxx xxxxxxxx");
}

#[test]
fn test_strong_inside_link() {
    let mut html = String::new();
    let mut html_file =
        File::open("../test-samples/dybr-bug-with-strong-inside-link.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert_that(&result).contains("[**Just God**](http://fanfics.me/ficXXXXXXX)");
}

#[test]
fn test_tables_with_newlines() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-tables-masked.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    let m = indoc! { "[![Embedded YouTube video](https://img.youtube.com/vi/ZZZZZZZZZ/0.jpg)](https://www.youtube.com/watch?v=ZZZZZZZZZ)\n|Maybe I'm foolish, maybe I'm blind\nThinking I can see through this and see what's behind\nGot no way to prove it so maybe I'm blind\nBut I'm only human after all,\nI'm only human after all\nDon't put your blame on me|xxxxx xxxx, x xxxxxx, xxxxx xxxx —xxxxxx\nxxx xxxxx, xxx xxxx xxxxxx xxxxxx xxx, x xxxxxx xxx xxx xx xxx\nxxxx x xxxx xx xxxx xxxxxxx xxxxxxxxxxxxx, xxx xxx xxxxxxxx, x xxxxxx.\nxx x xxxxx xxxx xxxxxxx, x xxxxx-xx xxxxxx,\nx xxxxx xxxx xxxxxxx, x xxxxx xxxxxx.\nxx xxxx xxxx|\n|||\n[xxxxxx xxxxx xxxxx x xxxxxxx](/)\nx xxxx xxxxxxxxx xxxxxxx xxxxxxxxxxx xx xxxx xxxxx. x xxxxx xxxxxxx, xxxx xxxxx xxxxxxx xx xxxxxxxxxx xxxxxx. xxx xxxxxxxx, xxx xxxxxxxxx xxxxxxxxxxxxxx xx xxxxx —xxxxxxxxxx xxxxxxxxxx x xxxxx xxxxxxxxxxxxx xxxxxxxxx. x xxx xxxxxxxxxxxx*xxxx*, xxxxxx xxxx, xxxxxxxxxx xxxxx xxxxxxxx, xxxxxxxxxx x xxxxxxxxx. xx xxxxxx xxxxx xxxxxxxxxxxxxxxxx —x xxxxxx xxx xxxx.\nxxxxx xxxxxxxxxx xxxxx x xxxx xxxxxxxxxx xxxxx. xxxxx. x xxxxx: «x xxxxxx xxxxxxx, x xxxxx xxx xxxx, xx xxxxxxxx xxxxxx», —xxx xxxxx xxxxxxxx. xxxxxx xxx x xxxx xxxx xxxxxxxx xxxxxxxx xxxxxxx xxxx xxxxxxxxxxx xxxxxxxxxx, xxxxxxx xxxxxx xxxxxx xxx xxxxx, xxxxxxxxxxx x x xxxxxxx xxxxxxxxx.\nxx x xxxxx xxxx xxxxxxx. xxxxxx xxxxx? xxxxxxxxxxx x xxxxxxxxx xxxxxx.\nx xxxxx x xxxxxxxxxx x xxxxx... x xxxxxx xxxx xxxxxx xxxxxxx xxxxxxxx. xx xxxx, x xxxxxx xxx-xx xxxxxxxxx xx xxxxxxx, xxx xxxxxx xxxxxx, xxx xxx xxxxx, xxxxx xxxxxxxx xx xxxx... x xxxxxx xxxxxxx xx xxxx xxxxx, xxx, xxxxx xxxx xxxxxxxxxx, x xxxxx xxxxxxxxx xx xxxxx. x xxx-xx xxx xxxxx xxxxxxx xxxxxxxxxxxxx.\nxxxxxx xx... xx xxx xx xxxxxxxxxxxxx xxxxxx xxxxxxxxxxxxx x xxxxxxxxxx xxxxx, xxxxx xxx xxxx xxxxxxxxx, x xxxxx xxx xxxxxxxxx, xxx xxxxxxx xxx, xxx xxxx xxxxxxx xxxxxx, x xx xxx, xxx xxxx xxxxxxxx." };

    assert_that!(result).contains(m);
    // let result = rewrite_html(&html, false);
    // assert_that!(result).contains(m);
}

#[test]
fn test_tables_crash2() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-tables-2-masked.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let table_with_vertical_header = parse_html(&html, false);
    let m = indoc! {"xxxxx xxxxxxxxxx xxxxxxx x xxxxx))~~xxxxxxxx xxxxxxxx~~\n## At a Glance\n|Current Conditions:|Open all year. No reservations. No services.|\n|||\n| Reservations: | No reservations. |\n| Fees | No fee. |\n| Water: | No water. |"};

    assert_that!(table_with_vertical_header).contains(m);

    let table_with_vertical_header = rewrite_html(&html, false);

    let m = indoc! { "xxxxx xxxxxxxxxx xxxxxxx x xxxxx))~~xxxxxxxx xxxxxxxx~~\n## At a Glance\n|Current Conditions:|Open all year. No reservations. No services.|\nReservations:|No reservations.|\nFees|No fee.|\nWater:|No water.|"};

    assert_that!(table_with_vertical_header).contains(m);
}

#[test]
fn test_html_from_text() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/real-world-1.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");

    let mut tag_factory: HashMap<String, Box<dyn html2md::TagHandlerFactory>> = HashMap::new();
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
