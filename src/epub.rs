use serde::Deserialize;
/*
 * table contet
 * */

#[derive(Deserialize, Debug)]
struct Toc {
    #[serde(rename = "head")]
    head: Head,
    #[serde(rename = "docTitle")]
    doc_title: DocTitle,
    #[serde(rename = "navMap")]
    nav_map: NavMap,
}

#[derive(Deserialize, Debug)]
struct Head {
    #[serde(rename = "meta")]
    meta: Vec<Meta>,
}

#[derive(Deserialize, Debug)]
struct Meta {
    #[serde(rename = "@content")]
    content: String,
    #[serde(rename = "@name")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct DocTitle {
    #[serde(rename = "text")]
    text: String,
}

#[derive(Deserialize, Debug)]
pub struct NavMap {
    #[serde(rename = "navPoint")]
    nav_points: Vec<NavPoint>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
struct NavLabel {
    #[serde(rename = "text")]
    text: String,
}

#[derive(Deserialize, Debug)]
struct Content {
    #[serde(rename = "@src")]
    src: String,
}

/* content.opf */
#[derive(Deserialize, Debug)]
struct Package {
    manifest: Manifest,
    spine: Spine,
}

#[derive(Deserialize, Debug)]
struct Manifest {
    #[serde(rename = "item")]
    items: Vec<ManifestItem>,
}

#[derive(Deserialize, Debug)]
struct ManifestItem {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "@media-type")]
    media_type: String,
}

#[derive(Deserialize, Debug)]
struct Spine {
    #[serde(rename = "itemref")]
    items: Vec<ItemRef>,
}

#[derive(Deserialize, Debug)]
struct ItemRef {
    #[serde(rename = "@idref")]
    idref: String,
}

/* container.xml */
#[derive(Deserialize, Debug)]
struct Container {
    #[serde(rename = "rootfiles")]
    rootfiles: RootFiles,
}

#[derive(Deserialize, Debug)]
struct RootFiles {
    #[serde(rename = "rootfile")]
    rootfile: RootFile,
}

#[derive(Deserialize, Debug)]
struct RootFile {
    #[serde(rename = "@full-path")]
    full_path: String,
    #[serde(rename = "@media-type")]
    media_type: String,
}

pub mod epub {
    use crate::epub::Container;
    use crate::epub::ManifestItem;
    use crate::epub::NavMap;
    use crate::epub::Package;
    use crate::epub::Toc;
    use clap::error::Result;
    use scraper::Html;
    use scraper::Selector;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::io::Read;
    use std::path::Path;
    use zip::ZipArchive;

    pub fn load(path: &Path) -> HashMap<String, Vec<String>> {
        let files_map = import_data(path);
        // println!("{:?}", files_map.keys());
        let container_file = files_map
            .get("META-INF/container.xml")
            .expect("container.xml not found");

        let container = get_container(container_file).expect("container not found");
        /* get root file */
        let root_file = container.rootfiles.rootfile.full_path;

        let opf_file = files_map.get(&root_file).expect("content.opf not found");

        let base_dir = root_file.rfind('/').map(|i| &root_file[..=i]).unwrap_or("");
        let toc_key = format!("{}toc.ncx", base_dir);
        let toc_ncx = files_map.get(&toc_key).expect("toc.ncx not found");

        let nav_map = define_structure(toc_ncx).expect("nav map not found!");
        // some time toc.ncx does not sync with content.opf
        let chapters = define_chapters(opf_file).expect("chapters not found!");

        let extracted_content = merge(base_dir, nav_map, chapters, &files_map);
        return extracted_content;
    }

    fn define_chapters(content_opf: &str) -> Result<Vec<ManifestItem>, quick_xml::DeError> {
        let package: Package = quick_xml::de::from_str(content_opf)?;
        Ok(package.manifest.items)
    }

    fn get_container(container_file: &str) -> Result<Container, quick_xml::DeError> {
        let container: Container = quick_xml::de::from_str(container_file)?;
        Ok(container)
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

    fn define_structure(toc: &str) -> Result<NavMap, quick_xml::DeError> {
        let table_of_content: Toc = quick_xml::de::from_str(toc)?;
        Ok(table_of_content.nav_map)
    }

    /*
       merge content with chapter
       */

    fn merge(
        base_dir: &str,
        table_of_content: NavMap,
        chapters: Vec<ManifestItem>,
        files_map: &HashMap<String, String>,
    ) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        let is_equal = chapters.len() == table_of_content.nav_points.len();
        if is_equal {
            for nav_point in table_of_content.nav_points {
                if let Some(file_content) =
                    files_map.get(&format!("{}{}", base_dir, nav_point.content.src))
                {
                    result.insert(nav_point.nav_label.text, remove_tags(file_content));
                }
            }
        } else {
            // labeled chapters from toc
            let toc_srcs: HashSet<&str> = table_of_content
                .nav_points
                .iter()
                .map(|n| n.content.src.as_str())
                .collect();

            for nav_point in &table_of_content.nav_points {
                if let Some(file_content) =
                    files_map.get(&format!("{}{}", base_dir, nav_point.content.src))
                {
                    let cleaned = remove_tags(file_content);
                    result.insert(nav_point.nav_label.text.clone(), cleaned);
                }
            }

            // remaining chapters not in toc keyed by href
            for chapter in chapters {
                if chapter.media_type == "application/xhtml+xml"
                    && !toc_srcs.contains(chapter.href.as_str())
                {
                    if let Some(file_content) = files_map.get(&chapter.href) {
                        result.insert(chapter.href, remove_tags(file_content));
                    }
                }
            }
        }

        result
    }

    /*
       remove html tags
       */
    fn remove_tags(raw_content: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let document = Html::parse_document(raw_content);
        let body_selector = Selector::parse("body").unwrap();
        let p_selector = Selector::parse("h1, p").unwrap();
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
