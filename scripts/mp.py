import time
import mediapipe as mp
from mediapipe.tasks import python
from mediapipe.tasks.python import vision
import cv2
import numpy as np

cap = cv2.VideoCapture(0)

model_path = 'face_landmarker.task'

BaseOptions = mp.tasks.BaseOptions
FaceLandmarker = mp.tasks.vision.FaceLandmarker
FaceLandmarkerOptions = mp.tasks.vision.FaceLandmarkerOptions
VisionRunningMode = mp.tasks.vision.RunningMode

options = FaceLandmarkerOptions(
    base_options=BaseOptions(model_asset_path=model_path),
    running_mode=VisionRunningMode.IMAGE,
    output_face_blendshapes=True
)

def visualize(
    image,
    detection_result
) -> np.ndarray:
  """Draws bounding boxes and keypoints on the input image and return it.
  Args:
    image: The input RGB image.
    detection_result: The list of all "Detection" entities to be visualize.
  Returns:
    Image with bounding boxes.
  """
  annotated_image = image.copy()
  height, width, _ = image.shape

  for face in detection_result.face_landmarks:
    # draw the left and right eye pupils
    left_eye_idx, right_eye_idx = 468, 473

    left_eye = face[left_eye_idx]
    right_eye = face[right_eye_idx]

    # draw the left and right eye pupils
    cv2.circle(annotated_image, (int(left_eye.x * width), int(left_eye.y * height)), 5, (0, 255, 0), -1)
    cv2.circle(annotated_image, (int(right_eye.x * width), int(right_eye.y * height)), 5, (0, 255, 0), -1)

  return annotated_image

with FaceLandmarker.create_from_options(options) as landmarker:
    while cap.isOpened():
        success, image = cap.read()
        if not success:
            print("Ignoring empty camera frame.")
            continue

        image = cv2.cvtColor(cv2.flip(image, 1), cv2.COLOR_RGB2BGR)
        # Load the input image from a numpy array.
        mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=image)

        face_landmarker_result = landmarker.detect(mp_image)

        # Draw the face landmarks on the image.
        image_copy = np.copy(mp_image.numpy_view())
        annotated_image = visualize(image_copy, face_landmarker_result)
        rgb_annotated_image = cv2.cvtColor(annotated_image, cv2.COLOR_BGR2RGB)

        cv2.imshow('MediaPipe Face Detection', rgb_annotated_image)
        if cv2.waitKey(5) & 0xFF == 27:
            break
