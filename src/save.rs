use dirs_next::document_dir;
use docx::{document::Paragraph, Docx};

use crate::app::Days;

// TODO refactor this
pub fn save(days: &Days) {
    let mut docx = Docx::default();

    for day in days.days.iter() {
        let date = day.date.format("%Y-%m-%d").to_string();
        let para = Paragraph::default().push_text(date);
        docx.document.push(para);

        for note in day.content.iter() {
            let para = Paragraph::default().push_text(note.as_str());
            docx.document.push(para);
        }
        docx.document.push(Paragraph::default());
        docx.document.push(Paragraph::default());
    }

    let mut outdir = document_dir().unwrap().to_str().unwrap().to_owned();
    let now = chrono::Utc::now().naive_utc().format("%Y-%m-%d");
    outdir.push_str(format!("/eng-log-{}.docx", now).as_str());

    docx.write_file(outdir).unwrap();
}
