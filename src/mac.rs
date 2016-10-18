// Rust code generation
#[macro_export]
macro_rules! c {
    // Finished
    () => {};

    // Parse toplevel #include macros
    (#include < $i:ident .h> $($rest:tt)*) => {c!{$($rest)*}};
    (#include $l:tt $($rest:tt)*) => {c!{$($rest)*}};

    // Parse toplevel raw macros
    (raw $body:tt $($rest:tt)*) => {c!{$($rest)*}};

    // Parse parameters
    (C_PARAM $name:ident : $t:ty as $ct:tt) => {
        $name: $t
    };
    (C_PARAM $name:ident : $t:ty as $ct:tt , $($rest:tt)*) => {
        $name: $t ,
        c!{C_PARAM $($rest)*}
    };

    // Parse function declarations
    ($(#[$m:meta])*
     fn $id:ident ( $($name:ident : $t:ty as $ct:tt),* ) -> $rt:ty as $rct:tt $body:tt $($rest:tt)*) => {
        extern "C"
		{
            $(#[$m])*
            pub fn $id ( $($name : $t),* ) -> $rt ;
        }
        c!{$($rest)*}
    };
    ($(#[$m:meta])*
     fn $id:ident ( $($name:ident : $t:ty as $ct:tt),* ) $body:tt $($rest:tt)*) => {
        extern "C"
		{
            $(#[$m])*
            pub fn $id ( $($name : $t),* ) ;
        }
        c!{$($rest)*}
    };

    // Parse struct definiton
    ($(#[$m:meta])*
     struct $id:ident { $($i:ident : $t:ty as $c:tt ,)* } $($rest:tt)*) => {
        $(#[$m])*
        #[repr(C)]
        struct $id
		{
            $($i : $t ,)*
        }
        c!{$($rest)*}
    };

    // Parse enum definition
    ($(#[$m:meta])*
     enum $id:ident { $($i:ident ,)* } $($rest:tt)*) => {
        $(#[$m])*
        #[repr(C)]
        enum $id
		{
            $($i ,)*
        }
        c!{$($rest)*}
    };
}
