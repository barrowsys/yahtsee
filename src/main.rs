use lol_html::html_content::{ContentType, Element};
use lol_html::{HtmlRewriter, Settings};
use std::fs;
use std::io::{self, Read};

struct Template {
    html: String,
    args: Vec<String>,
    selector: lol_html::Selector,
}

impl Template {
    fn new(element: scraper::element_ref::ElementRef) -> Template {
        let arg_sel = scraper::Selector::parse("arg").unwrap();
        let mut args = Vec::<String>::new();
        for arg in element.select(&arg_sel) {
            args.push(String::from(arg.value().id().unwrap()));
        }
        let id = String::from(element.value().id().unwrap());
        Template {
            selector: id.parse().unwrap(),
            html: String::from(&element.inner_html()),
            args: args,
        }
    }
    fn get_rewriter(&self) -> (&lol_html::Selector, lol_html::ElementContentHandlers) {
        (
            &self.selector,
            lol_html::ElementContentHandlers::default().element(move |el| {
                let args: std::collections::HashMap<String, String> = el
                    .attributes()
                    .iter()
                    .map(|attr| (attr.name(), attr.value()))
                    .collect();
                let mut rewriters = vec![];
                let mut rewriters2 = vec![];
                // let mut selectors = vec![];
                for arg in self.args.iter() {
                    let repl = match args.get(arg) {
                        Some(v) => v,
                        None => continue,
                    };
                    let sel: lol_html::Selector = format!("arg#{}", arg).parse().unwrap();
                    // selectors.push(sel);
                    // let sel2: &lol_html::Selector = selectors.get(selectors.len() - 1).unwrap();
                    rewriters.push(sel);
                    rewriters2.push(lol_html::ElementContentHandlers::default().element(
                        move |el: &mut Element| {
                            el.replace(repl, ContentType::Html);
                            Ok(())
                        },
                    ));
                }
                let mut rw2 = vec![];
                for i in 0..rewriters.len() {
                    rw2.push((rewriters.get(i).unwrap(), rewriters2.remove(0)));
                }
                let mut content = vec![];
                let mut rw = HtmlRewriter::try_new(
                    Settings {
                        element_content_handlers: rw2,
                        ..Settings::default()
                    },
                    |c: &[u8]| content.extend_from_slice(c),
                )
                .unwrap();
                let selfhtml: Vec<u8> = self.html.bytes().collect();
                rw.write(&selfhtml).unwrap();
                rw.end().unwrap();
                el.replace(&String::from_utf8(content).unwrap(), ContentType::Html);
                Ok(())
            }),
        )
    }
}

fn main() {
    let mut raw_html = String::new();
    io::stdin().read_to_string(&mut raw_html).unwrap();
    let raw_html = fs::read_to_string("test.html").expect("Couldn't read file");
    let html = scraper::Html::parse_document(&raw_html);
    let selector = scraper::Selector::parse("template").unwrap();
    let mut templates = Vec::<Template>::new();

    for element in html.select(&selector) {
        templates.push(Template::new(element));
    }
    let handlers = templates.iter().map(|t| t.get_rewriter()).collect();
    let mut output = vec![];
    let mut rewriter = HtmlRewriter::try_new(
        Settings {
            element_content_handlers: handlers,
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    )
    .unwrap();
    let new_vec: Vec<u8> = raw_html.bytes().collect();
    rewriter.write(&new_vec).unwrap();
    rewriter.end().unwrap();
    println!("{}", String::from_utf8(output).unwrap());
}
