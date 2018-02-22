from __future__ import unicode_literals

import os
from _ctypes import Structure, POINTER
from contextlib import contextmanager
from ctypes import c_char_p, c_int32, cdll
from glob import glob

import ujson as json
from future.builtins import range

dylib_dir = os.path.join(os.path.dirname(__file__), "dylib")
dylib_path = glob(os.path.join(dylib_dir, "libsnips_nlu_ontology*"))[0]
lib = cdll.LoadLibrary(dylib_path)


@contextmanager
def string_array_pointer(ptr):
    try:
        yield ptr
    finally:
        lib.nlu_ontology_destroy_string_array(ptr)


@contextmanager
def string_pointer(ptr):
    try:
        yield ptr
    finally:
        lib.nlu_ontology_destroy_string(ptr)


@contextmanager
def builtin_entity_light_array(ptr):
    try:
        yield ptr
    finally:
        lib.nlu_ontology_destroy_builtin_light_entity_array(ptr)


class C_STRING_ARRAY(Structure):
    _fields_ = [
        ("data", POINTER(c_char_p)),
        ("size", c_int32)
    ]

    def to_dict(self):
        return [self.data[i].decode("utf8") for i in range(self.size)]


class LIGHT_BUILTIN_ENTITY(Structure):
    _fields_ = [
        ("entity", c_char_p),
        ("entity_kind", c_char_p),
        ("value", c_char_p),
        ("range_start", c_int32),
        ("range_end", c_int32)
    ]

    def to_dict(self):
        return {
            "entity": json.loads(self.entity),
            "entity_kind": self.entity_kind.decode("utf-8"),
            "value": self.value.decode("utf-8"),
            "range": {"start": self.range_start, "end": self.range_end},
        }


class LIGTH_BUILTIN_ENTITY_ARRAY(Structure):
    _fields_ = [
        ("data", POINTER(LIGHT_BUILTIN_ENTITY)),
        ("size", c_int32)
    ]

    def to_dict(self):
        return [self.data[i].to_dict() for i in range(self.size)]
