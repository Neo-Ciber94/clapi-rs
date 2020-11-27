use clapi::*;

fn main(){
    //trace_macros!(true);
    let app = app! { MyApp =>
        (@arg range =>
            (count => 1..10)
            (description => "hello desc")
            (values => 1, 2, 3)
            (default => 1)
        )
        (@option numbers =>
            (@arg N)
        )
        //(@arg value)
    };

    println!("{}", stringify!(hello world));

    //trace_macros!(false);


    println!("{:#?}", app);
}

/*
app! { myApp =>
    (handler (, args) => {

    })
    (option range =>
        (required => true)
        (arg min => (count => 1) (type => i64))
        (arg min => (count => 1) (type => i64))
    )
    (option mode =>
        (arg mode => (values => 1, 2, 3))
    )
}
*/