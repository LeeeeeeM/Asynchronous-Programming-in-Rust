// 在文件开头添加格式化相关的导入
use once_cell::sync::OnceCell;
use std::error::Error;
use std::fmt::{Display, Write as WriteFmt};
use std::fs::File;
use std::io::Write;

// 添加格式化相关的导入
use prettyplease::unparse;
use syn::parse_file;

const FN_KW: &str = "coroutine";
const W_KW: &str = "wait";
static L_TRM_LN: OnceCell<usize> = OnceCell::new();

fn l_trm_len() -> usize {
    *L_TRM_LN.get().expect("Line terminator len not set")
}

// 修改rewrite函数，添加格式化功能
pub fn rewrite(src: String, dest: File) -> Result<(), impl Display> {
    detect_line_ending(&src);
    let mut dest = dest;
    // Find the start point of async blocks
    let start_points = find_kw_start_points(&src);

    // No keywords, no async functions, do nothing
    if start_points.is_empty() {
        return Err(format!("No `{FN_KW}` function found."));
    }

    let mut async_start_end = vec![];

    for start in start_points {
        // find the end
        // Find the end of the async function
        let mut brackets_counter = 0;
        let mut end = start;

        for char in src[start..].chars() {
            end += 1;
            match char {
                '{' => brackets_counter += 1,
                '}' => {
                    brackets_counter -= 1;
                    if brackets_counter == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }

        // store the locations
        async_start_end.push((start, end));
    }

    // 创建一个字符串变量来收集所有内容
    let mut all_content = String::new();
    
    // 收集所有非协程函数的代码
    let mut pos_tracker = 0;
    for (start, end) in &async_start_end {
        all_content.push_str(&src[pos_tracker..*start]);
        pos_tracker = *end;
    }
    // 收集最后一个异步函数之后的代码
    all_content.push_str(&src[pos_tracker..]);

    // 转换并收集每个协程函数的代码
    for (i, (start, end)) in async_start_end.into_iter().enumerate() {
        let id = i.to_string();

        let async_fn = String::from(&src[start..end - 1]);

        // 转换协程函数
        let transformed = transform(&async_fn, &id);

        // 收集转换后的协程函数代码
        all_content.push_str(&transformed);
    }

    // 添加格式化逻辑
    match parse_file(&all_content) {
        Ok(ast) => {
            // 格式化代码
            let formatted_code = unparse(&ast);
            // 写入格式化后的代码
            dest.write_all(formatted_code.as_bytes()).unwrap();
        },
        Err(_) => {
            // 如果格式化失败，回退到原始转换的代码
            dest.write_all(all_content.as_bytes()).unwrap();
        }
    }

    Ok(())
}

fn find_kw_start_points(s: &str) -> Vec<usize> {
    let mut start_points = vec![];
    let mut index = 0;

    for line in s.lines() {
        // Remove everything that's commented
        let (txt, _) = match line.split_once("//") {
            Some((txt, commented)) => (txt, commented),
            None => (line, ""),
        };

        // Search in the text that's not commented
        match txt.find(FN_KW) {
            Some(kw_start) => {
                start_points.push(index + kw_start);
                index += line.len() + l_trm_len();
            }

            None => {
                // An empty line will only be a `\n`
                let len = if line.len() == 0 {
                    l_trm_len()
                } else {
                    line.len() + l_trm_len()
                };
                index += len;
            }
        }
    }
    start_points
}

// Transforms an async function into a state machine, "mimmicing"
// what happens when compiling an async function in Rust
fn transform(async_fn: &str, id: &str) -> String {
    // first Comment out the async function
    let commented = comment_orig(&async_fn);
    // Then  rewrite the async function itself
    let (args, new_async_fn) = create_new_async_fn(&async_fn, &id);
    // Rewrite the async function to a state machine
    let rewritten = rewrite_async_fn(&async_fn, &id, args).unwrap();
    format!("{commented}{new_async_fn}{rewritten}")
}

/// Format and comment out the original "async" function
fn comment_orig(orig: &str) -> String {
    let mut res = String::new();
    writeln!(
        &mut res,
        "

// =================================
// We rewrite this:
// =================================
    "
    )
    .unwrap();
    for line in orig.lines() {
        writeln!(&mut res, "// {line}").unwrap();
    }
    writeln!(
        &mut res,
        "
// }}

// =================================
// Into this:
// =================================
"
    )
    .unwrap();

    res
}

// Returns the new async function and it's arguments
fn create_new_async_fn(func: &str, coro_id: &str) -> (Vec<(String, String)>, String) {
    // first line is expected to be the function definition `keyword fn name() -> ReturnType`
    // remove the keyword
    let def = &func.lines().nth(0).unwrap()[FN_KW.len() + 1..];

    // get the name of the function
    let (fn_name, arg_start) = def.split_once("(").expect("Expected `(`").clone();
    let (args, _) = arg_start.split_once(")").expect("Expected `)`");

    let args = get_args(&args);

    let args_fmt = format_args_name_and_types(&args);
    let arg_names = if args.is_empty() {
        format!("()")
    } else {
        format_args_names_only(&args)
    };

    let new_async_fn = format!(
        "{fn_name}({args_fmt}) -> impl Future<Output=String> {{
    Coroutine{coro_id}::new{arg_names}
}}
        "
    );

    (args, new_async_fn)
}

/// Rewrite the async function (this is very brittle, but does
/// the job for our example)
fn rewrite_async_fn(
    s: &str,
    id: &str,
    args: Vec<(String, String)>,
) -> Result<String, Box<dyn Error>> {
    let w_kw_len = W_KW.len();

    // Store the code in each "step" in this variable
    let mut steps = vec![];
    // Store the future call that we yield on
    let mut futures = vec![];

    let mut buffer = String::new();
    // Skip the first line since that's the function definition
    for line in s.lines().skip(1) {
        // If the line contains the keyword it's an await-point
        if line.contains(W_KW) {
            // Store the steps since last await point as a "step"
            steps.push(buffer.clone());
            buffer.clear();
            // Remove the keyword itself (i.e. `.await`)
            let l = &line.trim_end()[..line.len() - 1 - w_kw_len - 1];
            // we need both the future call and the variable name since
            // we most likely reference this variable name in the next "step"
            // This could be:
            // `let txt = Http::get("...").await`
            // or simply
            // `join_all(futures).await`
            match l.split_once("=") {
                Some((var, fut)) => {
                    // This could fail in so many ways...
                    let varname = &var[var.find("let").unwrap() + 3..].trim();
                    futures.push((varname.to_string(), fut.to_string()));
                }
                None => futures.push(("_".to_string(), l.trim().to_string())),
            }

            // We store the variable name and the future as a tuple since they're connected
        } else {
            buffer.push_str(line);
            buffer.push_str("\n");
        }
    }

    steps.push(buffer);

    // Write our steps enum. We know it will start with "Start" and end with "Resolved"
    // but we need to add one step for each await point

    let step_args = format_args_types_only(&args);

    let mut steps_enum = format!(
        "
enum State{id} {{
    Start{step_args},"
    );

    // We only support this kind of future
    for i in 1..steps.len() {
        write!(
            &mut steps_enum,
            "
    Wait{i}(Box<dyn Future<Output = String>>),"
        )?;
    }

    write!(
        &mut steps_enum,
        "
    Resolved,
}}"
    )?;

    // So, our `State` enum is finished, we create a coroutine struct and a simple
    // `new` implementation
    let coro_args = format_args_name_and_types(&args);
    let coro_args_names = format_args_names_only(&args);

    let coroutine = format!(
        "
struct Coroutine{id} {{
    state: State{id},
}}

impl Coroutine{id} {{
    fn new({coro_args}) -> Self {{
        Self {{ state: State{id}::Start{coro_args_names} }}
    }}
}}
"
    );

    // This is our future implementation
    // NB! Notice how we force all futures to return a string even if they
    // don't (if not we this get's very complicated without type information available)
    let mut imp = format!(
        "
impl Future for Coroutine{id} {{
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {{
        loop {{"
    );

    for (i, step) in steps.iter().enumerate() {
        // This is the index for the next step in the state machine
        let next = i + 1;

        // We need to special case the first call since that
        // happens before we reach an `await` point

        // This will receive the input args to the function
        let impl_fut_first_args = format_args_names_only(&args);

        if i == 0 {
            // if there are no futures in this "coro" fn we only have Start -> Resolved
            if futures.is_empty() {
                write!(
                    &mut imp,
                    "
        match self.state {{
                State{id}::Start{impl_fut_first_args} => {{
                    // ---- Code you actually wrote ----
                {step}
                    // ---------------------------------
                    self.state = State{id}::Resolved;
                    break PollState::Ready(String::new());
                }}
"
                )?;
                continue;
            }

            let futname = &futures[i].1;
            write!(
                &mut imp,
                "
        match self.state {{
                State{id}::Start{impl_fut_first_args} => {{
                    // ---- Code you actually wrote ----
                {step}
                    // ---------------------------------
                    let fut{next} = Box::new({futname});
                    self.state = State{id}::Wait{next}(fut{next});
                }}
"
            )?;

        // These steps are await-points where we await a future
        } else if i < steps.len() - 1 {
            let varname = &futures[i - 1].0;
            let fut = &futures[i].1;
            write!(
                &mut imp,
                "
                State{id}::Wait{i}(ref mut f{i}) => {{
                    match f{i}.poll() {{
                        PollState::Ready({varname}) => {{
                            // ---- Code you actually wrote ----
                        {step}
                            // ---------------------------------
                            let fut{next} = Box::new({fut});
                            self.state = State{id}::Wait{next}(fut{next});
                        }}
                        PollState::NotReady => break PollState::NotReady,
                    }}
                }}
"
            )?;

        // This is the part after the last await point. There is no need to yield any more
        } else {
            let varname = &futures[i - 1].0;
            write!(
                &mut imp,
                "
                State{id}::Wait{i}(ref mut f{i}) => {{
                    match f{i}.poll() {{
                        PollState::Ready({varname}) => {{
                            // ---- Code you actually wrote ----
                        {step}
                            // ---------------------------------
                            self.state = State{id}::Resolved;
                            break PollState::Ready(String::new());
                        }}
                        PollState::NotReady => break PollState::NotReady,
                    }}
                }}
"
            )?;
        }
    }

    // If we poll the future after it has resolved, we panic
    writeln!(
        &mut imp,
        "
                State{id}::Resolved => panic!(\"Polled a resolved future\")
            }}
        }}
    }}
}}"
    )?;

    // Format the different parts of the Coroutine implementation to a string
    Ok(format!("{steps_enum}\n{coroutine}\n{imp}"))
}

// this expects something like `txt: String, i: usize` or an empty string
fn get_args(s: &str) -> Vec<(String, String)> {
    let mut res = vec![];
    if s.trim().is_empty() {
        return res;
    }

    let args = s.split(",");
    for arg in args {
        let (argname, ty) = arg.split_once(":").expect("Expected `:`");
        res.push((argname.trim().to_string(), ty.trim().to_string()));
    }

    res
}

/// Gets:
/// `&[(txt, String), (i: usize)]`
/// Outputs
/// `(String, usize)`
/// If there are no args it returns: ""
fn format_args_types_only(args: &[(String, String)]) -> String {
    if args.is_empty() {
        String::new()
    } else {
        let mut args_fmt: String = args.iter().map(|(_n, ty)| format!("{ty},")).collect();
        // remove last `,`
        args_fmt.pop();
        format!("({args_fmt})")
    }
}

/// Gets:
/// `&[(txt, String), (i, usize)]`
/// Outputs
/// `txt: String, i: usize`
/// If there are no args it returns: ""
fn format_args_name_and_types(args: &[(String, String)]) -> String {
    if args.is_empty() {
        String::new()
    } else {
        let mut args_fmt: String = args.iter().map(|(n, ty)| format!("{n}: {ty},")).collect();
        // remove last `,`
        args_fmt.pop();
        format!("{args_fmt}")
    }
}

/// Gets:
/// `&[(txt, String), (i: usize)]`
/// Outputs
/// `(txt, i)`
/// If there are no args it returns: ""
fn format_args_names_only(args: &[(String, String)]) -> String {
    if args.is_empty() {
        String::new()
    } else {
        let mut args_fmt: String = args.iter().map(|(n, _ty)| format!("{n},")).collect();
        // remove last `,`
        args_fmt.pop();
        format!("({args_fmt})")
    }
}

/// Since both text editors and github can change the line endings of files
/// we need to account for \n line endings on Windows as well as \r\n.
fn detect_line_ending(s: &str) {
    let mut chars = s.chars().into_iter().peekable();

    let len = loop {
        let current = chars.next();
        match chars.peek() {
            Some(&'\n') => {
                if let Some('\r') = current {
                    break 2;
                } else {
                    break 1;
                }
            }
            Some(_) => (),
            None => {
                // defaults if we for some reason fall through
                if cfg!(windows) {
                    break 2;
                } else {
                    break 1;
                }
            }
        }
    };
    L_TRM_LN.set(len).unwrap();
}