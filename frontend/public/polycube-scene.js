// <polycube-scene cells="[[x,y,z],...]"> : 3D view of a polycube.
// The Elm application owns the state; this element only draws what its
// `cells` attribute describes.
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

const FIRST_CELL_COLOUR = 0xb5651d;
const LAST_CELL_COLOUR = 0xf0c08a;
const EDGE_COLOUR = 0x3c2818;
const CUBE_SIDE = 0.96;
const FIELD_OF_VIEW_DEGREES = 40;
const CAMERA_DIRECTION = new THREE.Vector3(1.2, 0.9, 1.6).normalize();
const CAMERA_MARGIN = 1.15;

class PolycubeScene extends HTMLElement {
  static observedAttributes = ["cells"];

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

    this.camera = new THREE.PerspectiveCamera(FIELD_OF_VIEW_DEGREES, 1, 0.1, 500);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;

    this.resizeObserver = new ResizeObserver(() => this.fitToElement());
    this.resizeObserver.observe(this);
    this.fitToElement();
    this.showCells();
    this.renderer.setAnimationLoop(() => {
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

  attributeChangedCallback() {
    if (this.renderer) {
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
    for (const cube of [...this.cubes.children]) {
      this.cubes.remove(cube);
      cube.traverse((part) => {
        part.geometry?.dispose();
        part.material?.dispose();
      });
    }
  }

  showCells() {
    const cells = JSON.parse(this.getAttribute("cells") ?? "[]");
    this.clearCubes();
    cells.forEach((cell, placementIndex) => {
      this.cubes.add(buildCube(cell, placementIndex / Math.max(cells.length - 1, 1)));
    });
    this.frameCubes();
  }

  // Centres the polycube and backs the camera off far enough to see it all.
  frameCubes() {
    const bounds = new THREE.Box3().setFromObject(this.cubes);
    if (bounds.isEmpty()) {
      return;
    }
    const centre = bounds.getCenter(new THREE.Vector3());
    const radius = bounds.getSize(new THREE.Vector3()).length() / 2;
    const halfFieldOfView = THREE.MathUtils.degToRad(FIELD_OF_VIEW_DEGREES / 2);
    const distance = (radius / Math.sin(halfFieldOfView)) * CAMERA_MARGIN;
    this.controls.target.copy(centre);
    this.camera.position.copy(centre).addScaledVector(CAMERA_DIRECTION, distance);
  }
}

// `placementProgress` runs from 0 (first cell placed) to 1 (last one).
function buildCube([x, y, z], placementProgress) {
  const geometry = new THREE.BoxGeometry(CUBE_SIDE, CUBE_SIDE, CUBE_SIDE);
  const colour = new THREE.Color(FIRST_CELL_COLOUR).lerp(
    new THREE.Color(LAST_CELL_COLOUR),
    placementProgress,
  );
  const cube = new THREE.Mesh(geometry, new THREE.MeshStandardMaterial({ color: colour }));
  const edges = new THREE.LineSegments(
    new THREE.EdgesGeometry(geometry),
    new THREE.LineBasicMaterial({ color: EDGE_COLOUR }),
  );
  cube.add(edges);
  // The lattice has z upwards, Three.js has y upwards: a quarter turn about
  // x, not an axis swap, so that mirror images are not exchanged.
  cube.position.set(x, z, -y);
  return cube;
}

customElements.define("polycube-scene", PolycubeScene);
