;; Counter with global variable ;;
(module
  (import "msg" "reply"
    (func $msg_reply (param $nonce i64) (param i32) (param i32)))

  (func $read (param $nonce i64)
    (i32.store
      (i32.const 0)
      (global.get 0)
    )
    (call $msg_reply
      (local.get $nonce)
      (i32.const 0)
      (i32.const 4)
    )
  )

  (func $write (param $nonce i64)
    (global.set 0
      (i32.add
        (global.get 0)
        (i32.const 1)
      )
    )
  )

  ;; Both increments and reads
  (func $inc_read (param $nonce i64)
    (call $write (local.get $nonce))
    (call $read (local.get $nonce))
  )

  (memory $memory 1)
  (export "memory" (memory $memory))
  (global (mut i32) (i32.const 0))
  (export "canister_query read" (func $read))
  (export "canister_query inc_read" (func $inc_read))
  (export "canister_update write" (func $write))
)
