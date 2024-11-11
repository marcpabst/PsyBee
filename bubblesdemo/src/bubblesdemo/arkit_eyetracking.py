from rubicon.objc import ObjCClass, NSObject, send_super, ObjCInstance, objc_method, ObjCProtocol, objc_property, py_from_ns
import pandas as pd

try:
    ARSession = ObjCClass("ARSession")
    ARConfiguration = ObjCClass("ARConfiguration")
    ARFaceTrackingConfiguration = ObjCClass("ARFaceTrackingConfiguration")
    ARSessionDelegate = ObjCProtocol("ARSessionDelegate")
    ARFaceAnchor = ObjCClass("ARFaceAnchor")
    NSMutableArray = ObjCClass("NSMutableArray")
    NSNumber = ObjCClass("NSNumber")
    NSMutableDictionary = ObjCClass("NSMutableDictionary")
    NSLock = ObjCClass("NSLock")


    class FaceDistanceDelegate(NSObject, protocols=[ARSessionDelegate]):
        """
        Delegate class for ARSession, implementing the ARSessionDelegate protocol.
        Note that this class cannot include non-Objective-C types as properties.
        Also, this class must be instantiated with alloc().init().
        """

        data = objc_property()

        # lock to avoid race conditions
        lock = objc_property()

        @objc_method
        def init(self):
            self = ObjCInstance(send_super(__class__, self, 'init'))
            self.data = NSMutableDictionary.alloc().init()
            self.lock = NSLock.alloc().init()

            self.data["face_distances"] = NSMutableArray.alloc().init()
            self.data["timestamps"] = NSMutableArray.alloc().init()
            self.data["left_eye_rotation_x"] = NSMutableArray.alloc().init()
            self.data["left_eye_rotation_y"] = NSMutableArray.alloc().init()
            self.data["right_eye_rotation_x"] = NSMutableArray.alloc().init()
            self.data["right_eye_rotation_y"] = NSMutableArray.alloc().init()

            return self

        @objc_method
        def get_current_face_distance(self):
            """ Get the current face distance. """
            # # lock the data
            # self.lock.lock()

            # if there are no face distances, return None
            if len(self.data["face_distances"]) == 0:
                return None

            # otherwise, return the last face distance
            out = self.data["face_distances"][-1]

            # # unlock the data
            # self.lock.unlock()

            return out

        @objc_method
        def clear_all():
            """ Remove all face distances. """
            pass

        @objc_method
        def session_didUpdateFrame_(self, session, frame) -> None:

            for anchor in frame.anchors:
                # check if the anchor is a face anchor
                if isinstance(anchor, ARFaceAnchor):
                    # get the distance
                    dist = NSNumber.numberWithFloat_(anchor.getDistanceToScreen())
                    timestamp = NSNumber.numberWithDouble_(frame.timestamp)

                    left_eye_rot_xy = anchor.leftEyeXAndYRotations()
                    right_eye_rot_xy = anchor.rightEyeXAndYRotations()

                    self.data["face_distances"].addObject_(dist)
                    self.data["timestamps"].addObject_(timestamp)
                    self.data["left_eye_rotation_x"].addObject_(left_eye_rot_xy["x"])
                    self.data["left_eye_rotation_y"].addObject_(left_eye_rot_xy["y"])
                    self.data["right_eye_rotation_x"].addObject_(right_eye_rot_xy["x"])
                    self.data["right_eye_rotation_y"].addObject_(right_eye_rot_xy["y"])
                    return

        @objc_method
        def get_data(self):
            """ Get the data. """
            return self.data

    mode = "arkit"
except:
    print("Could not import ARKit classes. Using simulation mode.")
    mode = "sim"


class ARKitEyeTracker():
    def __init__(self):
        if mode == "sim": return
        self.ar_delegate = FaceDistanceDelegate.alloc().init()
        self.config = ARFaceTrackingConfiguration.alloc().init()
        self.ar_session = ARSession.alloc().init()
        self.ar_session.delegate = self.ar_delegate

    def run(self):
        """ Run the ARSession. """
        if mode == "sim": return
        self.ar_session.runWithConfiguration(self.config)

    def pause(self):
        """ Pause the ARSession. """
        if mode == "sim": return
        self.ar_session.pause()

    def get_current_face_distance(self):
        """ Get the current face distance. """
        if mode == "sim": return 0.3
        try:
            return float(py_from_ns(self.ar_delegate.get_current_face_distance()))
        except:
            return 0.0

    def get_face_distances(self):
        """ Get all face distances. This is wasteful... """
        return [float(py_from_ns(dist)) for dist in self.ar_delegate.face_distances]

    def get_timestamps(self):
        """ Get all timestamps (in seconds). """
        return [float(py_from_ns(ts)) for ts in self.ar_delegate.timestamps]

    def as_df(self):
        """ Get the face distances and timestamps as a pandas DataFrame. """
        data = self.ar_delegate.get_data()

        return pd.DataFrame({
            "timestamp": data["timestamps"],
            "face_distance": data["face_distances"],
            "left_eye_rotation_x": data["left_eye_rotation_x"],
            "left_eye_rotation_y": data["left_eye_rotation_y"],
            "right_eye_rotation_x": data["right_eye_rotation_x"],
            "right_eye_rotation_y": data["right_eye_rotation_y"]
        })

    def clear_all(self):
        """ Remove all face distances. """
        self.ar_delegate.clear_all()
