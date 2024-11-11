import pandas as pd
from tkinter.filedialog import askopenfilename
import lets_plot as gg
import scipy as sp
import numpy as np

filename1 = askopenfilename()
filename2 = askopenfilename()

dfs = pd.read_csv(filename1), pd.read_csv(filename2)
dfs_new = []

gg.LetsPlot.setup_show_ext()

def plot_histogram_and_quit(x):
    p = gg.ggplot(pd.DataFrame(x, columns=['sampling_rate'])) \
        + gg.geom_histogram(gg.aes(x='sampling_rate'), bins=100)

    p.show()
    quit()

for df in dfs:
    # zero the timestamp column
    df['timestamp'] = df['timestamp'] - df['timestamp'][0]

    # remove the 'distance' column
    df = df.drop(columns='face_distance')

    # substract the mean from right_eye_rotation_y and left_eye_rotation_y
    df = df.melt(id_vars='timestamp', var_name='variable', value_name='rotation')
    df[['eye', 'Direction']] = df['variable'].str.extract(r'^(left|right)_eye_rotation_(x|y)$')

    # z-score normalization
    def z_score_norm(x):
        return (x - x.mean()) / x.std()

    # high-pass filter with a cutoff frequency of 1 Hz
    def high_pass_filter(x):
        b, a = sp.signal.butter(1, .5, btype='high', fs=60)
        return sp.signal.filtfilt(b, a, x, padtype = 'even')

    # low-pass filter with a cutoff frequency of 15 Hz
    def low_pass_filter(x):
        b, a = sp.signal.butter(2, 10, btype='low', fs=60)
        return sp.signal.filtfilt(b, a, x)

    # calculate the first derivative
    def first_derivative(x):
        return np.gradient(x)


    df['rotation'] = df.groupby(['eye', 'Direction'])['rotation'].transform(high_pass_filter)
    # df['rotation'] = df.groupby(['eye', 'direction'])['rotation'].transform(low_pass_filter)

    # remove the first and last second of data
    df = df[df['timestamp'] > df['timestamp'].min() + 1]
    df = df[df['timestamp'] < df['timestamp'].max() - 1]


    df['rotation'] = df.groupby(['eye'])['rotation'].transform(z_score_norm)

    # filter for left eye
    df = df[df.eye == 'left']

    dfs_new.append(df)

df1, df2 = dfs_new

p1 = gg.ggplot(df1) \
    + gg.geom_line(gg.aes(x='timestamp', y='rotation', colour='Direction')) \
    + gg.xlab("Time (s)") \
    + gg.ylab("Displacement (z-scored)") \
    + gg.ggtitle("Vertical Stripe Movement")

p2 = gg.ggplot(df2) \
    + gg.geom_line(gg.aes(x='timestamp', y='rotation', colour='Direction')) \
    + gg.xlab("Time (s)") \
    + gg.ylab("Displacement (z-scored)") \
    + gg.ggtitle("Horizontal Stripe Movement")

p3 = gg.ggplot(df1) \

p = gg.gggrid([p1, p2], 1) + gg.ggsize(1000, 500)


p.show()
