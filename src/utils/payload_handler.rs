use actix_multipart::{Field, Multipart};
use actix_web::web;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str;

#[derive(Debug, Clone)]
pub struct UploadedFiles {
    pub name: String,
    pub path: String,
}
impl UploadedFiles {
    fn new(filename: &str) -> UploadedFiles {
        UploadedFiles {
            name: filename.to_string(),
            path: format!("./files/{}", filename),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Projects {
    title: String,
    description: String,
    homepage: String,
    repository: String,
    priority: u32,
    images: Vec<String>,
}

pub async fn split_payload(payload: &mut Multipart) -> (Projects, Vec<UploadedFiles>) {
    let mut files: Vec<UploadedFiles> = Vec::new();

    /* fill with default values for now */
    let mut form: Projects = Projects {
        title: "".to_string(),
        description: "".to_string(),
        homepage: "".to_string(),
        repository: "".to_string(),
        priority: 1,
        images: Vec::new(),
    };

    /* iterate over all form fields */
    while let Some(item) = payload.next().await {
        let mut field: Field = item.expect("split_payload err");
        let content_type = field.content_disposition().unwrap(); // should return form data, the field name and the file name if set
        let name = content_type.get_name().unwrap();
        /* let filename=content_type.get_filename();  can also be used to check and save all file fields */

        if name != "file" {
            // we will only save content from the "file" field tho
            while let Some(chunk) = field.next().await {
                let data = chunk.expect("split_payload err chunk");
                /* convert bytes to string and print it  (just for testing) */
                if let Ok(s) = str::from_utf8(&data) {
                    let data_string = s.to_string();
                    println!("{:?}", data_string);
                    /* all not file fields of your form (feel free to fix this mess) */
                    match name {
                        "title" => form.title = data_string,
                        "description" => form.description = data_string,
                        "homepage" => form.homepage = data_string,
                        "repository" => form.repository = data_string,
                        "priority" => form.repository = data_string.parse().expect("not a number"),
                        _ => println!("invalid field found"),
                    };
                };
            }
        } else {
            match content_type.get_filename() {
                Some(filename) => {
                    let file = UploadedFiles::new(filename); // create new UploadedFiles
                    let file_path = file.path.clone();
                    let mut f = web::block(move || std::fs::File::create(&file_path))
                        .await
                        .unwrap();  // create file at path
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        f = web::block(move || f.write_all(&data).map(|_| f))
                            .await
                            .unwrap(); // write data chunks to file
                    }
                    files.push(file.clone()); // files vec with all file data for testing
                    form.images.push(file.name); // form only needs name
                }
                None => {
                    println!("file none");
                }
            }
        }
    }
    (form, files)
}
