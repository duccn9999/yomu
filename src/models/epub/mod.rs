use serde::Deserialize;

/*
 * Epub model
 * */
#[derive(Default, Debug)]
pub struct Epub {
    xml: XmlDeclaration,
    package: Package,
}

#[derive(Default, Debug)]
struct XmlDeclaration {
    version: String,
    encoding: String,
}

#[derive(Default, Debug)]
struct Package {
    version: String,
    unique_identifier: String,
    metadata: Metadata,
    manifest: Manifest,
    spine: Spine,
    guides: Guides,
}

#[derive(Default, Debug)]
struct Metadata {
    language: String,
    title: String,
    creator: Creator,
    contributor: Contributor,
    identifier: Vec<Identifier>,
    dc_date: String,
    metas: Metas,
}

#[derive(Default, Debug)]
struct Creator {
    text: String,
    file_as: String,
    role: String,
}

#[derive(Default, Debug)]
struct Contributor {
    text: String,
    role: String,
}

#[derive(Default, Debug)]
struct Identifier {
    text: String,
    id: Option<String>,
    scheme: String,
}

#[derive(Default, Debug)]
struct Metas {
    meta: String,
}

#[derive(Default, Debug)]
struct Manifest {
    item: Vec<ManifestItem>,
}

#[derive(Default, Debug)]
struct ManifestItem {
    id: String,
    href: String,
    media_type: String,
}

#[derive(Default, Debug)]
struct Spine {
    itemref: Vec<SpineItemRef>,
    toc: String,
}

#[derive(Default, Debug)]
struct SpineItemRef {
    idref: String,
}

#[derive(Default, Debug)]
struct Guides {
    guide_type: String,
    title: String,
    href: String,
}

/*
 * table contet
 * */

#[derive(Deserialize)]
struct Toc {
    #[serde(rename = "head")]
    head: Head,
    #[serde(rename = "docTitle")]
    doc_title: DocTitle,
    #[serde(rename = "navMap")]
    nav_map: NavMap,
}

#[derive(Deserialize)]
struct Head {
    #[serde(rename = "meta")]
    meta: Vec<Meta>,
}

#[derive(Deserialize)]
struct Meta {
    #[serde(rename = "@content")]
    content: String,
    #[serde(rename = "@name")]
    name: String,
}

#[derive(Deserialize)]
struct DocTitle {
    #[serde(rename = "text")]
    text: String,
}

#[derive(Deserialize)]
struct NavMap {
    #[serde(rename = "navPoint")]
    nav_points: Vec<NavPoint>,
}

#[derive(Deserialize)]
struct NavPoint {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@playOrder")]
    play_order: u32,
    #[serde(rename = "navLabel")]
    nav_label: NavLabel,
    #[serde(rename = "content")]
    content: Content,
}

#[derive(Deserialize)]
struct NavLabel {
    #[serde(rename = "text")]
    text: String,
}

#[derive(Deserialize)]
struct Content {
    #[serde(rename = "@src")]
    src: String,
}

mod epub {
    use crate::common::common::File;
    use crate::epub::Epub;
    use crate::epub::Toc;
    use crate::models::epub::NavMap;
    use scraper::Html;
    use scraper::Selector;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io::Read;
    use std::path::Path;
    use std::rc::Rc;
    use zip::ZipArchive;
    impl File<Epub> for Epub {
        fn unzip(&self, path: &Path) -> Vec<String> {
            let mut files_map = import_data(path);
            let toc_ncx = files_map.get_mut("toc.ncx").expect("toc.ncx not found!");
            let nav_map = define_structure(toc_ncx);
            let raw_content = merge(files_map);
            let extracted_content = remove_tags(raw_content);

            extracted_content
        }
    }

    /*
     * Import the epub content into object
     */
    fn import_data(path: &Path) -> HashMap<String, String> {
        let zip_file = std::fs::File::open(path).unwrap();
        let mut archive = ZipArchive::new(zip_file).unwrap();
        let mut files_map = HashMap::<String, String>::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            // check if its directory, or its a css file
            if file.is_dir() {
                continue;
            }
            let mut content = String::with_capacity(file.size() as usize);
            let _ = file.read_to_string(&mut content);
            files_map.insert(file.name().to_string(), content);
        }
        files_map
    }

    fn define_structure(content: &str) -> Result<NavMap, quick_xml::DeError> {
        let table_of_content: Toc = quick_xml::de::from_str(content)?;
        Ok(table_of_content.nav_map)
    }

    fn merge(files_map: HashMap<String, String>) -> Rc<RefCell<String>> {
        let total = files_map.values().map(|v| v.len()).sum();
        let mut content = String::with_capacity(total);
        for k in files_map.values() {
            content.push_str(k);
        }

        Rc::new(RefCell::new(content))
    }

    /*
     * Remove html tags
     */
    fn remove_tags(raw_content: Rc<RefCell<String>>) -> Vec<String> {
        let mut lines = Vec::<String>::new();
        let document = Html::parse_document(&raw_content.borrow());
        let body_selector = Selector::parse("body").unwrap();
        let p_selector = Selector::parse("p").unwrap();

        if let Some(body) = document.select(&body_selector).next() {
            for el in body.select(&p_selector) {
                let text = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    lines.push(text);
                }
            }
        }
        lines
    }
}
