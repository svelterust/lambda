(defsystem "lambda"
  :depends-on ("cffi" "bordeaux-threads")
  :serial t
  :components ((:file "lambda")
               (:file "input")
               (:file "text")
               (:file "rect")
               (:file "image")))
