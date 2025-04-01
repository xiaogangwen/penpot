;; This Source Code Form is subject to the terms of the Mozilla Public
;; License, v. 2.0. If a copy of the MPL was not distributed with this
;; file, You can obtain one at http://mozilla.org/MPL/2.0/.
;;
;; Copyright (c) KALEIDOS INC

(ns app.render-wasm.mem
  (:require
   [app.render-wasm.helpers :as h]
   [app.render-wasm.wasm :as wasm]
   [goog.object :as gobj]))

(defn ptr8->ptr32
  [value]
  ;; Divides the value by 4
  (bit-shift-right value 2))

(defn ptr32->ptr8
  [value]
  ;; Multiplies by 4
  (bit-shift-left value 2))

(defn get-list-size
  "Returns the size of a list in bytes"
  [list list-item-size]
  (* list-item-size (count list)))

(defn alloc-bytes
  [size]
  (when (= size 0)
    (js/console.trace "tried to allocate 0 bytes"))
  (let [ptr (h/call wasm/internal-module "_alloc_bytes" size)]
    ptr))

(defn alloc-bytes-32
  [size]
  (when (= size 0)
    (js/console.trace "tried to allocate 0 bytes"))
  (let [ptr (h/call wasm/internal-module "_alloc_bytes" size)]
    (ptr8->ptr32 ptr)))

(defn get-heap-u8
  []
  (gobj/get ^js wasm/internal-module "HEAPU8"))

(defn get-heap-u32
  []
  (let [heap (gobj/get ^js wasm/internal-module "HEAPU32")]
    heap))

(defn get-heap-f32
  []
  (gobj/get ^js wasm/internal-module "HEAPF32"))
