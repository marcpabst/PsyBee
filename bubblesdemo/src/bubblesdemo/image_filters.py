import numpy as np
from numpy.fft import fft2, ifft2, fftshift, ifftshift


def log_gabor_filter(image, f0, sigma_f):
    """
    Filters a 2D array using a log-Gabor filter.

    Parameters:
    image   : 2D numpy array representing the image to be filtered.
    f0      : Center frequency of the log-Gabor filter (in cycles per pixel).
    sigma_f : Bandwidth of the filter in log-frequency space.

    Returns:
    filtered_image : 2D numpy array of the filtered image.
    fft_filtered_image : 2D numpy array of the filtered image in the frequency domain.
    log_gabor : 2D numpy array representing the log-Gabor filter.
    """
    rows, cols = image.shape

    # Compute frequency grids
    u = np.fft.fftfreq(cols)
    v = np.fft.fftfreq(rows)
    u = fftshift(u)
    v = fftshift(v)
    u, v = np.meshgrid(u, v)
    radius = np.sqrt(u**2 + v**2)
    radius[radius == 0] = 1e-10  # Avoid division by zero at the origin

    # Log-Gabor filter formula
    log_rad = np.log(radius / f0)
    log_gabor = np.exp(- (log_rad ** 2) / (2 * (np.log(sigma_f / f0) ** 2)))
    log_gabor[radius == 0] = 0  # Suppress the DC component

    # Apply the log-Gabor filter in the frequency domain
    image_fft = fft2(image)
    image_fft_shifted = fftshift(image_fft)
    filtered_fft = image_fft_shifted * log_gabor
    filtered_fft = ifftshift(filtered_fft)
    filtered_image = np.real(ifft2(filtered_fft))

    return filtered_image, filtered_fft, log_gabor
