(defsystem "lambda"
  :depends-on ("cffi" "bordeaux-threads")
  :serial t
  :components ((:file "package")
               (:module "src"
               :components ((:file "kernel")
                            (:file "input")
                            (:file "text")))))
