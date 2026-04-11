(defpackage :lambda
  (:use :cl)
  (:export
   ;; Kernel
   :run
   ;; Draw
   :rect :clear :flush :with-scene
   ;; Input - polling
   :poll-events :key-name
   ;; Input - event types
   :+key-down+ :+key-up+
   ;; Input - modifier bits
   :+mod-shift+ :+mod-ctrl+ :+mod-alt+ :+mod-super+
   ;; Input - key codes: letters
   :+key-a+ :+key-b+ :+key-c+ :+key-d+ :+key-e+ :+key-f+ :+key-g+
   :+key-h+ :+key-i+ :+key-j+ :+key-k+ :+key-l+ :+key-m+ :+key-n+
   :+key-o+ :+key-p+ :+key-q+ :+key-r+ :+key-s+ :+key-t+ :+key-u+
   :+key-v+ :+key-w+ :+key-x+ :+key-y+ :+key-z+
   ;; Input - key codes: digits
   :+digit-0+ :+digit-1+ :+digit-2+ :+digit-3+ :+digit-4+
   :+digit-5+ :+digit-6+ :+digit-7+ :+digit-8+ :+digit-9+
   ;; Input - key codes: common
   :+space+ :+enter+ :+escape+ :+backspace+ :+tab+ :+delete+
   :+insert+ :+home+ :+end+ :+page-up+ :+page-down+
   ;; Input - key codes: punctuation
   :+comma+ :+period+ :+slash+ :+semicolon+ :+quote+
   :+bracket-left+ :+bracket-right+ :+backslash+
   :+minus+ :+equal+ :+backquote+
   ;; Input - key codes: arrows
   :+arrow-up+ :+arrow-down+ :+arrow-left+ :+arrow-right+
   ;; Input - key codes: modifiers
   :+shift-left+ :+shift-right+ :+control-left+ :+control-right+
   :+alt-left+ :+alt-right+ :+super-left+ :+super-right+
   ;; Input - key codes: F-keys
   :+f1+ :+f2+ :+f3+ :+f4+ :+f5+ :+f6+
   :+f7+ :+f8+ :+f9+ :+f10+ :+f11+ :+f12+
   ;; Input - key codes: misc
   :+caps-lock+ :+num-lock+ :+scroll-lock+ :+print-screen+ :+pause+))
