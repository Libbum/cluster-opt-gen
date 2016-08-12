use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::env;

fn read_cluster() -> Result<String, std::io::Error> {
    let mut f = try!(File::open("cluster.xyz"));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));  // `s` contains the contents of "cluster.xyz"
    Ok(s)
}

fn main() {
    //By default we generate values for node 1, although we can use a CLA to build other nodes (ultimately we need 1 - 30).
    let mut node = 1;
    if let Some(arg1) = env::args().nth(1) {
        node = arg1.parse().unwrap();
    }

    let numchunks = 24; //How many chunks? Historically we've used 12 but with optimisation I've decided 24 may be better.
    let cpus = 30;
    let numxi = 300;
    let numyi = 300;
    let numzi = 300;
    let a = 0.0035;

    let cluster = read_cluster().unwrap();

    println!("Building input files for node: {}", node);

    let a2 = a/2.0;
    let distnumz = numzi/cpus;
    let numx = numxi as f32;
    let numy = numyi as f32;
    let numz = numzi as f32;
    let grx = numx*a2-a2;
    let gry = numy*a2-a2;
    let grz = numz*a2-a2;

    //We have to generate the specific delocal oxygen line for all runs first, then spit later.
    let mut oline: Vec<String> = Vec::new();
    for xx in 0..numxi+5+1 {
        for yy in 0..numyi+5+1 {
            for zz in 0..distnumz+5+1 {
                let tx = -(grx+3.0*a)+(xx as f32)*(2.0*grx)/(numx-1.0);
                let ty = -(gry+3.0*a)+(yy as f32)*(2.0*gry)/(numy-1.0);
                let tz = -(grz+3.0*a)+((zz as f32)+((node as f32)-1.0)*(distnumz as f32))*(2.0*grz)/(numz-1.0);
                let current = format!("O   {:.5}   {:.5}   {:.5}", tx, ty, tz);
                oline.push(current);
            }
        }
    }

    //Cut into chuncks and write files.
    let cutsize = oline.len()/numchunks;
    let mut lowidx = 0;
    let mut highidx = cutsize;
    for it in 1..numchunks+1 {
        println!("Building chunk {}", it);
        let dirname = format!("chunk{:02}",it);
        match std::fs::create_dir(&dirname) {
            Err(why) => println!("Cannot create directory {}: {:?}", dirname, why.kind()),
            Ok(_) => {},
        }
        let infilename = format!("{}/input.gin", &dirname);
        let path = Path::new(&infilename);
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why.description()),
            Ok(file) => file,
        };
        match file.write_all(b"conp opti\n") {
                Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
                Ok(_) => {},
        }

        for line in lowidx..highidx {
            let system = "cart\n".to_string() + &cluster + &oline[line] + "\nlibrary streitzmintmire\n\n";
            match file.write_all(system.as_bytes()) {
                    Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
                    Ok(_) => {},
            }
        }
        lowidx = lowidx + cutsize;
        highidx = highidx + cutsize;
    }
}
