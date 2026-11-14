// <polycube-scene cells="[[x,y,z],...]" shown="n"> : 3D view of a polycube.
// `shown` limits the view to the first n cells; without it all are shown.
// `candidates` and `discarded` list cells drawn around the polycube: those
// waiting to extend it, and those it may no longer hold. `added` lists cells
// drawn as cubes of another colour: an extension of the polycube.
// `extent="n"` frames
// the room every polycube of at most n cells fits in, so that the view stays
// still while the polycube changes. `framed="[[x,y,z],[x,y,z]]"` frames the
// box between two opposite corner cells instead, for cells that spread far.
// The Elm application owns the state; this element only draws what its
// attributes describe.
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

const FIRST_CELL_COLOUR = 0xb5651d;
const LAST_CELL_COLOUR = 0xf0c08a;
const EDGE_COLOUR = 0x3c2818;
const CUBE_SIDE = 0.96;
const CANDIDATE_COLOUR = 0x2f8f6b;
const CANDIDATE_SIDE = 0.7;
const CANDIDATE_OPACITY = 0.35;
const DISCARDED_COLOUR = 0xa3261a;
const DISCARDED_SIDE = 0.45;
const ADDED_COLOUR = 0x3b6fb5;
// Share of the room of all polycubes that the camera frames: most polycubes
// stay near the first cell, so framing the whole room would shrink them;
// only the straightest ones graze the border.
const FRAMED_ROOM_SHARE = 0.6;
// Share of the sphere around a `framed` box that the camera frames: cells
// seldom reach the corners of the box, so the whole sphere would shrink them.
const FRAMED_BOX_SHARE = 0.7;
const FIELD_OF_VIEW_DEGREES = 40;
const CAMERA_DIRECTION = new THREE.Vector3(1.2, 0.9, 1.6).normalize();
const CAMERA_MARGIN = 1.15;
const APPEARANCE_MILLISECONDS = 260;
const SMALLEST_VISIBLE_SCALE = 0.001;

class PolycubeScene extends HTMLElement {
  static observedAttributes = [
    "cells",
    "shown",
    "candidates",
    "discarded",
    "added",
    "extent",
    "framed",
  ];

  connectedCallback() {
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.appendChild(this.renderer.domElement);

    this.scene = new THREE.Scene();
    this.scene.add(new THREE.AmbientLight(0xffffff, 1.4));
    const sun = new THREE.DirectionalLight(0xffffff, 2.2);
    sun.position.set(3, 6, 4);
    this.scene.add(sun);
    this.cubes = new THREE.Group();
    this.scene.add(this.cubes);
    this.cubeByKey = new Map();

    this.camera = new THREE.PerspectiveCamera(FIELD_OF_VIEW_DEGREES, 1, 0.1, 500);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;

    this.resizeObserver = new ResizeObserver(() => this.fitToElement());
    this.resizeObserver.observe(this);
    this.fitToElement();
    this.showCells();
    this.renderer.setAnimationLoop(() => {
      this.growAppearingCubes();
      this.controls.update();
      this.renderer.render(this.scene, this.camera);
    });
  }

  disconnectedCallback() {
    this.renderer.setAnimationLoop(null);
    this.resizeObserver.disconnect();
    this.controls.dispose();
    this.clearCubes();
    this.renderer.dispose();
    this.renderer.domElement.remove();
    this.renderer = null;
  }

  attributeChangedCallback(name) {
    if (!this.renderer) {
      return;
    }
    if (name === "shown") {
      this.showFirstCubes();
    } else {
      this.showCells();
    }
  }

  fitToElement() {
    const width = Math.max(this.clientWidth, 1);
    const height = Math.max(this.clientHeight, 1);
    this.renderer.setSize(width, height);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
  }

  clearCubes() {
    for (const key of [...this.cubeByKey.keys()]) {
      this.removeCube(key);
    }
  }

  removeCube(key) {
    const cube = this.cubeByKey.get(key);
    this.cubeByKey.delete(key);
    this.cubes.remove(cube);
    cube.traverse((part) => {
      part.geometry?.dispose();
      part.material?.dispose();
    });
  }

  cellsOf(attributeName) {
    return JSON.parse(this.getAttribute(attributeName) ?? "[]");
  }

  // Cubes already drawn are kept, so that only the cells that change move
  // on screen: a key names a cell and how it is drawn.
  showCells() {
    const placedCells = this.cellsOf("cells");
    const extent = Number(this.getAttribute("extent") ?? placedCells.length);
    const lastPlacementIndex = Math.max(extent - 1, 1);
    const builderByKey = new Map();
    placedCells.forEach((cell, placementIndex) => {
      builderByKey.set(`placed ${placementIndex} ${cell}`, () =>
        buildPlacedCube(cell, placementIndex, placementIndex / lastPlacementIndex),
      );
    });
    for (const cell of this.cellsOf("candidates")) {
      builderByKey.set(`candidate ${cell}`, () => buildCandidateCube(cell));
    }
    for (const cell of this.cellsOf("added")) {
      builderByKey.set(`added ${cell}`, () => buildAddedCube(cell));
    }
    for (const cell of this.cellsOf("discarded")) {
      builderByKey.set(`discarded ${cell}`, () => buildDiscardedCube(cell));
    }

    for (const key of [...this.cubeByKey.keys()]) {
      if (!builderByKey.has(key)) {
        this.removeCube(key);
      }
    }
    for (const [key, buildCube] of builderByKey) {
      if (!this.cubeByKey.has(key)) {
        const cube = buildCube();
        this.cubeByKey.set(key, cube);
        this.cubes.add(cube);
      }
    }
    this.frame();
    this.showFirstCubes();
  }

  // Hides the cubes beyond the `shown` attribute; a cube that becomes
  // visible grows from nothing so that the eye catches where it lands.
  showFirstCubes() {
    const shownAttribute = this.getAttribute("shown");
    const shownCount = shownAttribute === null ? Infinity : Number(shownAttribute);
    this.cubes.children.forEach((cube) => {
      const placementIndex = cube.userData.placementIndex;
      const isShown = placementIndex === undefined || placementIndex < shownCount;
      if (isShown && !cube.visible) {
        cube.userData.appearedAt = performance.now();
        cube.scale.setScalar(SMALLEST_VISIBLE_SCALE);
      }
      cube.visible = isShown;
    });
  }

  growAppearingCubes() {
    const now = performance.now();
    for (const cube of this.cubes.children) {
      if (cube.userData.appearedAt === undefined) {
        continue;
      }
      const progress = Math.min((now - cube.userData.appearedAt) / APPEARANCE_MILLISECONDS, 1);
      cube.scale.setScalar(Math.max(progress, SMALLEST_VISIBLE_SCALE));
      if (progress === 1) {
        delete cube.userData.appearedAt;
      }
    }
  }

  // Centres what must be seen and backs the camera off far enough to see it
  // all. With an `extent` the camera is set once and then left to the user.
  frame() {
    const extentAttribute = this.getAttribute("extent");
    const framedAttribute = this.getAttribute("framed");
    // A fixed frame is set once per value, then the camera is the user's.
    const fixedFrame = framedAttribute ?? extentAttribute;
    if (fixedFrame !== null && fixedFrame === this.fixedFrame) {
      return;
    }
    this.fixedFrame = fixedFrame;
    let bounds;
    if (framedAttribute !== null) {
      bounds = boxBetweenCells(JSON.parse(framedAttribute));
    } else if (extentAttribute !== null) {
      bounds = roomOfPolycubes(Number(extentAttribute));
    } else {
      bounds = new THREE.Box3().setFromObject(this.cubes);
    }
    if (bounds.isEmpty()) {
      return;
    }
    const centre = bounds.getCenter(new THREE.Vector3());
    let framedShare = 1;
    if (framedAttribute !== null) {
      framedShare = FRAMED_BOX_SHARE;
    } else if (extentAttribute !== null) {
      framedShare = FRAMED_ROOM_SHARE;
    }
    const radius = (bounds.getSize(new THREE.Vector3()).length() / 2) * framedShare;
    const halfFieldOfView = THREE.MathUtils.degToRad(FIELD_OF_VIEW_DEGREES / 2);
    const distance = (radius / Math.sin(halfFieldOfView)) * CAMERA_MARGIN;
    this.controls.target.copy(centre);
    this.camera.position.copy(centre).addScaledVector(CAMERA_DIRECTION, distance);
  }
}

// Room swept by the polycubes of at most `cellCount` cells: the first cell
// sits at the origin, no cell lies below it, the others reach at most
// `cellCount - 1` steps away. Given in scene axes.
function roomOfPolycubes(cellCount) {
  const reach = cellCount - 1 + CUBE_SIDE / 2;
  return new THREE.Box3(
    new THREE.Vector3(-reach, -CUBE_SIDE / 2, -reach),
    new THREE.Vector3(reach, reach, reach),
  );
}

// Box holding every cube between two opposite corner cells, in the scene's
// axes: see `placeInScene` for the turn from lattice to scene.
function boxBetweenCells(cornerCells) {
  const box = new THREE.Box3();
  for (const [x, y, z] of cornerCells) {
    box.expandByPoint(new THREE.Vector3(x, z, -y));
  }
  return box.expandByScalar(CUBE_SIDE / 2);
}

// `placementProgress` runs from 0 (first cell placed) to 1 (last one).
function buildPlacedCube(cell, placementIndex, placementProgress) {
  const colour = new THREE.Color(FIRST_CELL_COLOUR).lerp(
    new THREE.Color(LAST_CELL_COLOUR),
    placementProgress,
  );
  const cube = buildCube(cell, CUBE_SIDE, new THREE.MeshStandardMaterial({ color: colour }));
  cube.add(buildEdges(cube.geometry, EDGE_COLOUR));
  cube.userData.placementIndex = placementIndex;
  return cube;
}

// A cell of an extension of the polycube, told apart by its colour.
function buildAddedCube(cell) {
  const cube = buildCube(cell, CUBE_SIDE, new THREE.MeshStandardMaterial({ color: ADDED_COLOUR }));
  cube.add(buildEdges(cube.geometry, EDGE_COLOUR));
  return cube;
}

// A cell waiting to extend the polycube: a small see-through cube.
function buildCandidateCube(cell) {
  const material = new THREE.MeshStandardMaterial({
    color: CANDIDATE_COLOUR,
    transparent: true,
    opacity: CANDIDATE_OPACITY,
    depthWrite: false,
  });
  const cube = buildCube(cell, CANDIDATE_SIDE, material);
  cube.add(buildEdges(cube.geometry, CANDIDATE_COLOUR));
  return cube;
}

// A cell the polycube may no longer hold: an empty cage.
function buildDiscardedCube(cell) {
  const geometry = new THREE.BoxGeometry(DISCARDED_SIDE, DISCARDED_SIDE, DISCARDED_SIDE);
  const cage = buildEdges(geometry, DISCARDED_COLOUR);
  geometry.dispose();
  placeInScene(cage, cell);
  return cage;
}

function buildCube(cell, side, material) {
  const cube = new THREE.Mesh(new THREE.BoxGeometry(side, side, side), material);
  placeInScene(cube, cell);
  return cube;
}

function buildEdges(geometry, colour) {
  return new THREE.LineSegments(
    new THREE.EdgesGeometry(geometry),
    new THREE.LineBasicMaterial({ color: colour }),
  );
}

// Starts hidden: `showFirstCubes` decides what is seen and makes it grow.
function placeInScene(object, [x, y, z]) {
  // The lattice has z upwards, Three.js has y upwards: a quarter turn about
  // x, not an axis swap, so that mirror images are not exchanged.
  object.position.set(x, z, -y);
  object.visible = false;
}

customElements.define("polycube-scene", PolycubeScene);
