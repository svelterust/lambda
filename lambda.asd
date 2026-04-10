(defsystem "lambda"
		   :depends-on ("cffi")
		   :serial t
		   :components ((:file "package")
						(:module "src"
								 :components ((:file "kernel")))))
