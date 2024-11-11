# Modern Graphics Pipelines

<div class="grid cards" markdown>
- Traditional Graphics Pipeline
    ```mermaid
    graph TD
        A[Application] -->|Submit Draw Calls| B[Acquire Back Buffer]
        B -->|Render to| C[Back Buffer]
        C -->|Swap Buffers| D[Front Buffer]
        D -->|Display| E[Screen]
    ```
- Modern Graphics Pipeline
    
    ```mermaid
    graph TD
        A[Application] -->|Submit Draw Calls| B[Acquire Image]
            B -->|Record Commands| C[Command Buffer]
                C -->|Submit to| D[Graphics Queue]
                    D -->|Execute Commands| E[GPU]
                        E -->|Present Image| F[Presentation Engine]
                            F -->|Display| G[Screen]
    ```
</div>