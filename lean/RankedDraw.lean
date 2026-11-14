/-!
# Drawing by rank in a finite tree

Formal counterpart of the theorem of `docs/preuves/tirage-uniforme.md`.

A tree stands for the enumeration tree cut at the wanted size: its leaves
are the polycubes of that size, its inner nodes the smaller polycubes. An
inner node may have no leaf below it.

`Tree.pick` is the descent of `RankedPolycubes::polycube_at`: at each node it
enters the first child whose subtree holds the rank, after subtracting the
leaf counts of the children it skips. The main theorem, `Tree.pick_eq`, says
that this descent returns the leaf of that rank in left-to-right order. The
corollaries follow: every rank below the leaf count gives a leaf, every leaf
is given by some rank, and distinct ranks give distinct leaves when the
leaves are distinct. A uniform rank therefore gives a uniform leaf.

No dependency beyond Lean's core library.
-/

/-- A finite tree with values at its leaves. -/
inductive Tree (α : Type) where
  | leaf (value : α)
  | node (children : List (Tree α))

namespace Tree

variable {α : Type}

mutual
  /-- Leaves of the tree, from left to right. -/
  def leaves : Tree α → List α
    | leaf value => [value]
    | node children => leavesOfAll children

  /-- Leaves of a list of trees, from left to right. -/
  def leavesOfAll : List (Tree α) → List α
    | [] => []
    | child :: siblings => child.leaves ++ leavesOfAll siblings
end

mutual
  /-- Number of leaves, computed without listing them: `completion_count`. -/
  def leafCount : Tree α → Nat
    | leaf _ => 1
    | node children => leafCountOfAll children

  /-- Number of leaves of a list of trees. -/
  def leafCountOfAll : List (Tree α) → Nat
    | [] => 0
    | child :: siblings => child.leafCount + leafCountOfAll siblings
end

mutual
  /-- Descent by rank: `polycube_at`. -/
  def pick : Tree α → Nat → Option α
    | leaf value, rank => if rank = 0 then some value else none
    | node children, rank => pickAmong children rank

  /-- Enters the first tree whose leaves hold the rank. -/
  def pickAmong : List (Tree α) → Nat → Option α
    | [], _ => none
    | child :: siblings, rank =>
      if rank < child.leafCount then child.pick rank
      else pickAmong siblings (rank - child.leafCount)
end

mutual
  /-- The count used by the descent is the number of leaves. -/
  theorem leafCount_eq : (tree : Tree α) → tree.leafCount = tree.leaves.length
    | leaf _ => by simp [leafCount, leaves]
    | node children => by
      simp only [leafCount, leaves]
      exact leafCountOfAll_eq children

  theorem leafCountOfAll_eq :
      (trees : List (Tree α)) → leafCountOfAll trees = (leavesOfAll trees).length
    | [] => by simp [leafCountOfAll, leavesOfAll]
    | child :: siblings => by
      simp only [leafCountOfAll, leavesOfAll, List.length_append]
      rw [leafCount_eq child, leafCountOfAll_eq siblings]
end

mutual
  /-- The descent by rank returns the leaf of that rank. -/
  theorem pick_eq : (tree : Tree α) → (rank : Nat) → tree.pick rank = tree.leaves[rank]?
    | leaf value, rank => by
      cases rank <;> simp [pick, leaves]
    | node children, rank => by
      simp only [pick, leaves]
      exact pickAmong_eq children rank

  theorem pickAmong_eq :
      (trees : List (Tree α)) → (rank : Nat) → pickAmong trees rank = (leavesOfAll trees)[rank]?
    | [], rank => by simp [pickAmong, leavesOfAll]
    | child :: siblings, rank => by
      simp only [pickAmong, leavesOfAll]
      by_cases isInChild : rank < child.leafCount
      · simp only [isInChild, ↓reduceIte]
        rw [pick_eq child rank]
        rw [leafCount_eq] at isInChild
        exact (List.getElem?_append_left isInChild).symm
      · simp only [isInChild, ↓reduceIte]
        rw [pickAmong_eq siblings (rank - child.leafCount)]
        rw [leafCount_eq] at isInChild ⊢
        exact (List.getElem?_append_right (Nat.le_of_not_lt isInChild)).symm
end

/-- Every rank below the leaf count gives a leaf of the tree. -/
theorem pick_mem (tree : Tree α) (rank : Nat) (isBelowCount : rank < tree.leafCount) :
    ∃ value, tree.pick rank = some value ∧ value ∈ tree.leaves := by
  rw [leafCount_eq] at isBelowCount
  refine ⟨tree.leaves[rank], ?_, List.getElem_mem isBelowCount⟩
  rw [pick_eq, List.getElem?_eq_getElem isBelowCount]

/-- A rank that is not below the leaf count gives nothing. -/
theorem pick_none (tree : Tree α) (rank : Nat) (isBeyondCount : tree.leafCount ≤ rank) :
    tree.pick rank = none := by
  rw [leafCount_eq] at isBeyondCount
  rw [pick_eq, List.getElem?_eq_none isBeyondCount]

/-- Every leaf is given by some rank below the leaf count. -/
theorem pick_surjective (tree : Tree α) (value : α) (isLeaf : value ∈ tree.leaves) :
    ∃ rank, rank < tree.leafCount ∧ tree.pick rank = some value := by
  obtain ⟨rank, isBelowCount, isAtRank⟩ := List.getElem_of_mem isLeaf
  refine ⟨rank, ?_, ?_⟩
  · rw [leafCount_eq]; exact isBelowCount
  · rw [pick_eq, List.getElem?_eq_getElem isBelowCount, isAtRank]

/-- When the leaves are distinct, distinct ranks give distinct leaves. -/
theorem pick_injective (tree : Tree α) (leavesAreDistinct : tree.leaves.Nodup)
    (first second : Nat) (firstIsBelowCount : first < tree.leafCount)
    (secondIsBelowCount : second < tree.leafCount)
    (givesSameLeaf : tree.pick first = tree.pick second) : first = second := by
  rw [leafCount_eq] at firstIsBelowCount secondIsBelowCount
  rw [pick_eq, pick_eq, List.getElem?_eq_getElem firstIsBelowCount,
    List.getElem?_eq_getElem secondIsBelowCount] at givesSameLeaf
  have leavesAreEqual := Option.some.inj givesSameLeaf
  have differAtDistinctRanks := List.pairwise_iff_getElem.mp leavesAreDistinct
  rcases Nat.lt_trichotomy first second with isBefore | isSame | isAfter
  · exact absurd leavesAreEqual
      (differAtDistinctRanks first second firstIsBelowCount secondIsBelowCount isBefore)
  · exact isSame
  · exact absurd leavesAreEqual.symm
      (differAtDistinctRanks second first secondIsBelowCount firstIsBelowCount isAfter)

end Tree
