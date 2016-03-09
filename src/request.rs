/// Passed to server as handler

use std::fs;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::str;

use mime::Attr;
use hyper::header::{ContentDisposition, DispositionType, ContentType,
                    DispositionParam, Charset, ContentLength, Location};

use hyper::server::{Handler, Request, Response};
use hyper::status::StatusCode;
use hyper::{Get, Post};

use directory::Directory;
use ui;
use stream;

///
///
///
pub struct RequestHandler {
    verbose:   bool,
    directory: Directory,
}


impl RequestHandler {


    pub fn new(dir: Directory, verbose: bool) -> RequestHandler {
        RequestHandler {
            verbose:   verbose,
            directory: dir,
        }
    }


    fn handle_get(&self, req: Request, mut res: Response) {
        let resources = self.directory.list_available_resources();
        let uri = req.uri.to_string();

        if self.verbose {
            println!("Receiving a GET request from {} for {}",
                     req.remote_addr.to_string(),
                     uri);
        }

        if uri == "/" {
            res.send(ui::render_ui(&resources).as_bytes()).unwrap();
            return;
        }

        let mut name: Vec<u8> = Vec::new();
        name.extend_from_slice(uri[1..uri.len()].as_bytes());

        if resources.contains(&uri) {
            let path = self.directory.full_path(uri);
            let meta = fs::metadata(&*path).unwrap();
            let mut file: File = File::open(&*path).unwrap();
            let len = meta.len() as usize;

            res.headers_mut().set(ContentLength(len as u64));

            res.headers_mut().set(ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(
                    Charset::Iso_8859_1,
                    None,
                    name)]});

            let mut stream = res.start().unwrap();
            let mut buffer: [u8; 1024] = [0; 1024];
            let mut read_total: usize = 0;
            let mut sent_total: usize = 0;

            while read_total < len {
                let read: usize = file.read(&mut buffer).unwrap();
                let sent: usize = stream.write(&buffer[0 .. read]).unwrap();
                read_total = read_total + read;
                sent_total = sent_total + sent;
            }
            stream.end();

            if sent_total != read_total {
                println!("");
            }
        }
    }


    fn get_filename_from_form(&self, form: &str) -> Result<String, String> {
        let file_name = "filename=";
        let mut name  = "".to_string();

        for s in form.split(" ") {
            if s.contains(file_name) {
                let tmp: Vec<&str> = s.split("\"").collect();
                if tmp.len() != 2 && tmp[0] != file_name {
                    return Err("Error: Malformed form".to_string());
                }
                name = tmp[1].to_string();
            }
        }
        if name == "" {
            return Err("Error: File name not found".to_string());
        }
        Ok(name)
    }

    // FIXME: stream reading buffer should be the same as the body reader

    /// Parses the form part of the stream and returns the name of the file.
    ///
    /// reads the stream until it finds the filename
    fn parse_post_form(&self, req: &mut Request) -> Result<String, String> {
        const MAX_LEN: usize = 512;

        let mut tmp_buff: [u8; 1] = [0; 1];
        let mut buff: [u8; MAX_LEN] = [0; MAX_LEN];
        let mut read_total: usize = 0;
        let mut end_reached = false;

        // Read the form part of the stream
        while !end_reached {
            if read_total >= MAX_LEN {
                return Err(format!("Error: Post form is too long > {}", MAX_LEN));
            }
            let read = req.read(&mut tmp_buff).unwrap();
            // Stream end is reached before the form end
            if read < 1 {
                return Err("Error: Malformed form".to_string());
            }
            // Check if two consecutive new lines have been read
            if read_total > 4 &&
                tmp_buff[0]          == ('\n' as u8) &&
                buff[read_total - 1] == ('\r' as u8) &&
                buff[read_total - 2] == ('\n' as u8) &&
                buff[read_total - 3] == ('\r' as u8 )
            {
                end_reached = true;
            }
            buff[read_total] = tmp_buff[0].clone();
            read_total = read_total + read;
        }

        // Stringify the form buffer
        let form_raw = str::from_utf8(&buff[0..read_total]);
        let form;

        match form_raw {
            Err(e) => return Err(e.to_string()),
            Ok (f) => {
                if f.len() < 50 { // totaly arbitrary
                    return Err("Error: Malformed form".to_string())
                }
                form = f
            }
        }

        self.get_filename_from_form(form)
    }

    /// Return status code
    fn handle_post(&self, mut req: Request, mut res: Response) -> Result<usize, String> {
        let uri  = req.uri.to_string();
        let addr = req.remote_addr.to_string();

        if self.verbose {
            println!("Receiving a POST request from {}", addr);
        }
        if uri != "/" {
            return Err("Invalid request uri".to_string());
        }

        // advance to filename with no write stream
        // stream::advance_stream(input, no_stream, 5000, "filename="")

        // read filename
        // stream::write_stream(input, someting, 3000, "\"")

        // advance to file start
        // stream::advance_stream(input, no_stream, "\r\n\r\n")

        // stream::write_stream(req, file, file_length, boundary);


        //   let boundary  = req.headers.get::<ContendBoundary>();


        let file_name = self.parse_post_form(&mut req).unwrap();

        // Borrow scope
        {let stat: &mut StatusCode = res.status_mut();
         *stat = StatusCode::Found;}

        res.headers_mut().set(Location("/".to_string()));
        res.send(b"Something").unwrap();
        println!("Sending status code {}", StatusCode::Found.to_string());

        Ok(0)
    }


    fn handle_requests(&self, req: Request, res: Response) {
        match req.method {
            Post => {
                match self.handle_post(req, res) {
                    Ok (n)   => return,
                    Err(err) => println!("Error: {:?}", err)
                }
            },
            Get => {
                self.handle_get(req, res);
            },
            _ => return
        }
    }
}


impl Handler for RequestHandler {
    fn handle (&self, req: Request, res: Response) {
        self.handle_requests(req, res);
    }
}
