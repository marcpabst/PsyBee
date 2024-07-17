## Gradients

- use ramp texturese (https://stackoverflow.com/questions/47376499/creating-a-gradient-color-in-fragment-shader)



Introduce `Drawable`, `Shaders`, `Primitive`, `Uniform` and `Texture`:
- A `Drawable` is an object that can be drawn. It can be either a `Primitive`, a `Text`, or a `UIElement`.
  - A `Primitive` defines an object that is drawn using a speicfic `Shader` - it stores both a `VertexList`', a shader-specific `Uniform`, and (optionally) a `Texture`.
    - A `Shader` defines how a `Primitive` is to be drawn. It defines what vertex and pixel shaders to use and is in charge of the actual rendering within a `RenderPass`.
    - A `Texture` describes a texture buffer (containing pixel image data) in GPU memory. It can be bound by a `Shader`.
    - A `Uniform` describes data that should be made available to the `Shader` during rendering. 
- A `UIElement` describes a UI element that will be rendered to the screen.