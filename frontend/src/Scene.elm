module Scene exposing (Colouring(..), view, viewAmongNeighbours, viewColouredFramed, viewFirstCells)

{-| The 3D view of a polycube, drawn by the `polycube-scene` custom element
(`public/polycube-scene.js`).
-}

import Html exposing (Html)
import Html.Attributes as Attr
import Json.Encode as Encode
import Polycube exposing (Cell, Polycube)


view : Polycube -> Html msg
view polycube =
    Html.node "polycube-scene" [ cellsAttribute polycube ] []


{-| Shows only the first cells of the polycube, framed as if all were there,
so that the view does not move while the polycube grows.
-}
viewFirstCells : Int -> Polycube -> Html msg
viewFirstCells shownCellCount polycube =
    Html.node "polycube-scene"
        [ cellsAttribute polycube
        , Attr.attribute "shown" (String.fromInt shownCellCount)
        ]
        []


{-| Shows a polycube with the cells waiting to extend it, those it may no
longer hold and those of an extension, framed for the largest polycube to come, so that the view
does not move from one polycube to the next.
-}
viewAmongNeighbours :
    { largestCellCount : Int, candidates : List Cell, discarded : List Cell, added : List Cell }
    -> Polycube
    -> Html msg
viewAmongNeighbours neighbours polycube =
    Html.node "polycube-scene"
        [ Attr.attribute "extent" (String.fromInt neighbours.largestCellCount)
        , cellsAttribute polycube
        , encodedCellsAttribute "candidates" neighbours.candidates
        , encodedCellsAttribute "discarded" neighbours.discarded
        , encodedCellsAttribute "added" neighbours.added
        ]
        []


{-| How the cells of a framed view are coloured. `ByCopy n` : the cells list
copies of one polycube, `n` cells each, and every copy gets its own colour.
`ByLineage n parents` : copies again, but each takes after the colour of its
parent, whose birth rank `parents` gives. `ByIdentity n identities shapes` :
copies again, told apart by their identity so that each stays in place while
others come and go, and coloured after their shape: one colour per shape. `ByAge n` : the cells come in birth order, `n` at most,
and run through the rainbow.
-}
type Colouring
    = ByCopy Int
    | ByLineage Int (List Int)
    | ByIdentity Int (List Int) (List Int)
    | ByAge Int


{-| Shows cells in a view framed once on the box between two opposite
corner cells, so that it stays still while the cells spread.
-}
viewColouredFramed : Colouring -> { lowestCorner : Cell, highestCorner : Cell } -> List Cell -> Html msg
viewColouredFramed colouring frame cells =
    Html.node "polycube-scene"
        (encodedCellsAttribute "framed" [ frame.lowestCorner, frame.highestCorner ]
            :: cellsAttribute cells
            :: colouringAttributes colouring
        )
        []


colouringAttributes : Colouring -> List (Html.Attribute msg)
colouringAttributes colouring =
    case colouring of
        ByCopy cellsPerCopy ->
            [ copySizeAttribute cellsPerCopy ]

        ByLineage cellsPerCopy parentOfCopy ->
            [ copySizeAttribute cellsPerCopy, encodedIntsAttribute "copy-parents" parentOfCopy ]

        ByIdentity cellsPerCopy identities shapes ->
            [ copySizeAttribute cellsPerCopy
            , encodedIntsAttribute "copy-ids" identities
            , encodedIntsAttribute "copy-shapes" shapes

            -- Polycubes come and go too fast for cubes to grow into place.
            , Attr.attribute "instant" ""
            ]

        ByAge largestCellCount ->
            Attr.attribute "age-span" (String.fromInt largestCellCount)
                :: (if largestCellCount > largestCellCountDrawnOneByOne then
                        [ Attr.attribute "crowd" "" ]

                    else
                        []
                   )


{-| Beyond that many cells the element draws them all as one mesh, without
their edges: a mesh per cube would bring the view to a halt.
-}
largestCellCountDrawnOneByOne : Int
largestCellCountDrawnOneByOne =
    1500


copySizeAttribute : Int -> Html.Attribute msg
copySizeAttribute cellsPerCopy =
    Attr.attribute "copy-size" (String.fromInt cellsPerCopy)


encodedIntsAttribute : String -> List Int -> Html.Attribute msg
encodedIntsAttribute name ints =
    Attr.attribute name (Encode.encode 0 (Encode.list Encode.int ints))


cellsAttribute : Polycube -> Html.Attribute msg
cellsAttribute polycube =
    encodedCellsAttribute "cells" polycube


encodedCellsAttribute : String -> List Cell -> Html.Attribute msg
encodedCellsAttribute name cells =
    Attr.attribute name (Encode.encode 0 (Polycube.encodeCells cells))
