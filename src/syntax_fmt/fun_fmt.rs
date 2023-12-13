use std::fmt;  
  
struct CodeBlock<T> {  
    content: &'static str,  
    kind: &'static str,  
    acquire: &'static str,  
    read: &'static str,  
    write: &'static str,  
    other: T,  
}  
  
impl<T: std::fmt::Display> fmt::Display for CodeBlock<T> {  
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {  
        write!(f, "fun {}() acquires {} reads {} writes {} reads {}", self.content, self.acquire, self.read, self.write, self.other)  
    }  
}

#[test]
fn test_rewrite_fun_header_1() {
    let code = CodeBlock {  
        content: "f_multiple",  
        kind: "",  
        acquire: "R",  
        read: "R",  
        write: "T, S",  
        other: "reads G<u64>",  
    };  
    println!("{}", code);  
}