(module 
 (func $main 
 (result i32)
 (i32.const 1)
 )
 (memory $0 1)
 (data (i32.const 1) "11Hello world")
 (export "_start" (func $main))
 )