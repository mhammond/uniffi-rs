import threading

class ConcurrentHandleMap:
    """
    A map where inserting, getting and removing data is synchronized with a lock.
    """

    def __init__(self):
        # type Handle = int
        self._left_map = {}  # type: Dict[Handle, (Any, ctypes.c_void_p)]
        self._right_map = {}  # type: Dict[Any, (Handle, ctypes.c_void_p)]

        self._lock = threading.Lock()


    def insert(self, obj, arc_maker):
        with self._lock:
            if obj in self._right_map:
                return self._right_map[obj]
            else:
                handle = self._current_handle
                self._current_handle += self._stride
                arc = arc_maker(handle)
                self._left_map[handle] = (obj, arc)
                self._right_map[obj] = (handle, arc)
                return arc

    def get(self, handle):
        with self._lock:
            return self._left_map.get(handle)

    def remove(self, handle):
        with self._lock:
            if handle in self._left_map:
                (obj, arc) = self._left_map.pop(handle)
                del self._right_map[obj]
                return obj

# Magic number for the Rust proxy to call using the same mechanism as every other method,
# to free the callback once it's dropped by Rust.
IDX_CALLBACK_FREE = 0

class FfiConverterCallbackInterface:
    _handle_map = ConcurrentHandleMap()

    def __init__(self, cb, arc_maker):
        self._foreign_callback = cb
        self._arc_maker = arc_maker

    def drop(self, handle):
        self.__class__._handle_map.remove(handle)

    @classmethod
    def lift(cls, arc):
        obj = cls._handle_map.get(handle)[0]
        if not obj:
            raise InternalError("The object in the handle map has been dropped already")

        return obj

    @classmethod
    def read(cls, buf):
        handle = buf.readU64()
        cls.lift(handle)

    @classmethod
    def lower(cls, cb):
        arc = cls._handle_map.insert(cb, self._arc_maker)
        return arc

    @classmethod
    def write(cls, cb, buf):
        buf.writeU64(cls.lower(cb))
