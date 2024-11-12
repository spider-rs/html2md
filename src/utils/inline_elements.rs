use html5ever::LocalName;
use std::collections::HashSet;

lazy_static::lazy_static! {
    pub(crate) static ref LOCAL_NAMES: HashSet<LocalName> = {
        let mut set = HashSet::new();
        set.insert(html5ever::local_name!("a"));
        set.insert(html5ever::local_name!("abbr"));
        set.insert(html5ever::local_name!("acronym"));
        set.insert(html5ever::local_name!("audio"));
        set.insert(html5ever::local_name!("b"));
        set.insert(html5ever::local_name!("bdi"));
        set.insert(html5ever::local_name!("bdo"));
        set.insert(html5ever::local_name!("big"));
        set.insert(html5ever::local_name!("br"));
        set.insert(html5ever::local_name!("button"));
        set.insert(html5ever::local_name!("canvas"));
        set.insert(html5ever::local_name!("cite"));
        set.insert(html5ever::local_name!("code"));
        set.insert(html5ever::local_name!("data"));
        set.insert(html5ever::local_name!("datalist"));
        set.insert(html5ever::local_name!("del"));
        set.insert(html5ever::local_name!("dfn"));
        set.insert(html5ever::local_name!("em"));
        set.insert(html5ever::local_name!("embed"));
        set.insert(html5ever::local_name!("i"));
        set.insert(html5ever::local_name!("iframe"));
        set.insert(html5ever::local_name!("img"));
        set.insert(html5ever::local_name!("input"));
        set.insert(html5ever::local_name!("ins"));
        set.insert(html5ever::local_name!("kbd"));
        set.insert(html5ever::local_name!("label"));
        set.insert(html5ever::local_name!("map"));
        set.insert(html5ever::local_name!("mark"));
        set.insert(html5ever::local_name!("meter"));
        set.insert(html5ever::local_name!("noscript"));
        set.insert(html5ever::local_name!("object"));
        set.insert(html5ever::local_name!("output"));
        set.insert(html5ever::local_name!("picture"));
        set.insert(html5ever::local_name!("progress"));
        set.insert(html5ever::local_name!("q"));
        set.insert(html5ever::local_name!("ruby"));
        set.insert(html5ever::local_name!("s"));
        set.insert(html5ever::local_name!("samp"));
        set.insert(html5ever::local_name!("script"));
        set.insert(html5ever::local_name!("select"));
        set.insert(html5ever::local_name!("slot"));
        set.insert(html5ever::local_name!("small"));
        set.insert(html5ever::local_name!("span"));
        set.insert(html5ever::local_name!("strong"));
        set.insert(html5ever::local_name!("sub"));
        set.insert(html5ever::local_name!("sup"));
        set.insert(html5ever::local_name!("svg"));
        set.insert(html5ever::local_name!("template"));
        set.insert(html5ever::local_name!("textarea"));
        set.insert(html5ever::local_name!("time"));
        set.insert(html5ever::local_name!("tt"));
        set.insert(html5ever::local_name!("u"));
        set.insert(html5ever::local_name!("var"));
        set.insert(html5ever::local_name!("video"));
        set.insert(html5ever::local_name!("wbr"));
        set
    };

    pub(crate) static ref SKIP_ELEMENTS: HashSet<LocalName> = {
        let mut set = HashSet::new();
        set.insert(html5ever::local_name!("head"));
        set.insert(html5ever::local_name!("script"));
        set.insert(html5ever::local_name!("style"));
        set.insert(html5ever::local_name!("nav"));
        set
    };

    pub(crate) static ref BR: LocalName = html5ever::local_name!("br");

    pub(crate) static ref TABLE_ELEMENTS: HashSet<String> = {
        let mut set = HashSet::new();
        set.insert("table".to_string());
        set.insert("caption".to_string());
        set.insert("colgroup".to_string());
        set.insert("col".to_string());
        set.insert("thead".to_string());
        set.insert("tbody".to_string());
        set.insert("tfoot".to_string());
        set.insert("tr".to_string());
        set.insert("td".to_string());
        set.insert("th".to_string());
        set
    };
}
