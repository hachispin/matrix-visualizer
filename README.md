# matrix-visualizer

For visualizing matrices and transformations in both 2D and 3D.

See the site on: <https://hachispin.github.io/matrix-visualizer/>

As of 20/10/2026, looks like this:

![Screenshot of theprogram, showing the grid and sidebar](./showcase.png)

## Controls

- Scroll or pinch to change the magnification (zoom level) of the graph.
- Drag with the left mouse button to change the graph's orientation.
- For moving the focus point, type or tap `x`, `y` or `z`. This
  moves the focus point by a grid square in the specified positive
  axis. Press with SHIFT or "Reverse" to go in the opposite direction.
- Enter `~` or tap "Recenter" to reset the focus point back to the origin, (0, 0, 0).

---

<details>
<summary>Checklist</summary>

- [x] Implement the grid
  - [x] Decide on what type of camera to use
  - [x] Draw 2D/3D grid lines
  - [x] Add labels (x, y, z), numbers and arrowheads
  - [x] Make the spacing between grid lines based on magnification
  - [ ] Allow for a “2D mode” that locks the camera to the x-y plane

- [x] Make input boxes
  - [x] How should matrices be entered?
  - [x] A “transform” button for matrices (that affects the grid)
  - [x] Different types of boxes (matrix, point, vector, polygon) <!-- No vectors though. -->
  - [ ] Consider accessibility?

- [x] Draw points/vectors that the user enters
  - [x] User should be able to hide them
  - [x] Each drawn point/vector color-coded to its respective input box
  - [x] Enclosed area by a polygon should be shaded

- [x] Transforming
  - [ ] Toggle for whether the grid itself is affected by transformations
  - [ ] Should the camera “follow” the transformation?
  - [x] Animated with some easing function

</details>
