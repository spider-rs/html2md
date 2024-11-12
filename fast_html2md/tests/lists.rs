use html2md::parse_html;
use pretty_assertions::assert_eq;

#[test]
fn test_list_simple() {
    let md = parse_html(
        r#"<p><ul><li>Seven things has lady Lackless</li><li>Keeps them underneath her black dress</li><li>One a thing that's not for wearing</li></ul></p>"#,
        false,
    );
    assert_eq!(
        md,
        "\n\n* Seven things has lady Lackless\n* Keeps them underneath her black dress\n* One a thing that's not for wearing\n\n"
    )
}

#[test]
fn test_list_formatted() {
    // let's use some some broken html
    let md = parse_html(
        r#"
        <ul><p>
            <li>You should NEVER see this error
                <ul>
                    <li>Broken lines, broken strings
                    <li>Broken threads, broken springs</li>
                    <li>Broken idols, broken heads
                    <li>People sleep in broken beds</li>
                </ul>
            </li>
            <li>Ain't no use jiving</li>
            <li>Ain't no use joking</li>
            <li>EVERYTHING IS BROKEN
    "#,
        false,
    );
    assert_eq!(
        md,
        "\n\n* You should NEVER see this error\n  * Broken lines, broken strings\n  * Broken threads, broken springs\n  * Broken idols, broken heads\n  * People sleep in broken beds\n  \n* Ain't no use jiving\n* Ain't no use joking\n* EVERYTHING IS BROKEN"
    )
}

#[test]
fn test_list_stackedit() {
    let md = parse_html(
        r#"
    <ul>
        <li>
            <p>You should NEVER see this error</p>
            <ul>
                <li>
                <p>Broken lines, broken strings</p>
                </li>
                <li>
                <p>Broken threads, broken springs</p>
                </li>
                <li>
                <p>Broken idols, broken heads</p>
                </li>
                <li>
                <p>People sleep in broken beds</p>
                </li>
            </ul>
            </li>
            <li>
            <p>Ain’t no use jiving</p>
            </li>
            <li>
            <p>Ain’t no use joking</p>
            </li>
            <li>
            <p>EVERYTHING IS BROKEN</p>
            </li>
    </ul>"#,
        false,
    );
    assert_eq!(
        md,
        "* You should NEVER see this error\n  \n  * Broken lines, broken strings\n    \n  * Broken threads, broken springs\n    \n  * Broken idols, broken heads\n    \n  * People sleep in broken beds\n    \n  \n* Ain’t no use jiving\n  \n* Ain’t no use joking\n  \n* EVERYTHING IS BROKEN"
    )
}

#[test]
fn test_list_stackedit_add_brs() {
    let md = parse_html(
        r#"
    <ul>
        <li>
            <p>You should NEVER see this error</p>
            <ul>
                <li>
                <p>Broken lines, broken strings</p>
                </li>
                <li>
                <p>Broken threads, broken springs</p>
                </li>
                <li>
                <p>Broken idols, broken heads</p>
                </li>
                <li>
                <p>People sleep in broken beds</p>
                <br/>
                <br/>
                </li>
            </ul>
            </li>
            <li>
            <p>Ain’t no use jiving</p>
            </li>
            <li>
            <p>Ain’t no use joking</p>
            </li>
            <li>
            <p>EVERYTHING IS BROKEN</p>
            </li>
    </ul>"#,
        false,
    );
    assert_eq!(
        md,
        "* You should NEVER see this error\n  \n  * Broken lines, broken strings\n    \n  * Broken threads, broken springs\n    \n  * Broken idols, broken heads\n    \n  * People sleep in broken beds\n    \n    \n    \n  \n* Ain’t no use jiving\n  \n* Ain’t no use joking\n  \n* EVERYTHING IS BROKEN"
    )
}

#[test]
fn test_list_multiline() {
    let md = parse_html(
        r#"
        <ol>
            <li>
                <p>In the heat and the rains</p>
                <p>With whips and chains</p>
                <p>Just to see him fly<br/>So many die!</p>
            
            </li>
        </ol>
    "#,
        false,
    );
    assert_eq!(
        md,
        "1. In the heat and the rains\n   \n   With whips and chains\n   \n   Just to see him fly\n   So many die!"
    )
}

#[test]
fn test_list_multiline_formatted() {
    // let's use some some broken html
    let md = parse_html(
        r#"
        <ul><p>
            <li>You should NEVER see this error
                <ul>
                    <li>Broken lines, broken strings
                    <li>Broken threads, broken springs</li>
                    <li>Broken idols, broken heads
                    <li>People sleep in broken beds</li>
                    <li>
                        <p>Ain't no use jiving</p>
                        <p>Ain't no use joking</p>
                        <p>EVERYTHING IS BROKEN</p>
                    </li>
                </ul>
            </li>
    "#,
        false,
    );
    assert_eq!(
        md,
        "\n\n* You should NEVER see this error\n  * Broken lines, broken strings\n  * Broken threads, broken springs\n  * Broken idols, broken heads\n  * People sleep in broken beds\n  * Ain't no use jiving\n    \n    Ain't no use joking\n    \n    EVERYTHING IS BROKEN"
    )
}

#[test]
fn test_list_ordered() {
    // let's use some some broken html
    let md = parse_html(
        r#"
        <ol>
            <li>Now did you read the news today?</li>
            <li>They say the danger's gone away</li>
            <li>Well I can see the fire still alight</li>
            <li>Burning into the night</li>
        </ol>
    "#,
        false,
    );
    assert_eq!(
        md,
        "\
1. Now did you read the news today?
2. They say the danger's gone away
3. Well I can see the fire still alight
4. Burning into the night"
    )
}

#[test]
fn test_list_text_prevsibling() {
    let md = parse_html(
        r#"
        Phrases to describe me:
        <ul>
            <li>Awesome</li>
            <li>Cool</li>
            <li>Awesome and cool</li>
            <li>Can count to five</li>
            <li>Learning to count to six B)</li>
        </ul>
    "#,
        false,
    );
    assert_eq!(
        md,
        "\
Phrases to describe me:

* Awesome
* Cool
* Awesome and cool
* Can count to five
* Learning to count to six B)"
    )
}
