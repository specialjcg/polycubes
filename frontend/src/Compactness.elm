module Compactness exposing (describe)

{-| How compact a polycube is, told by its bounding box: the fuller the
box, the more compact the polycube.
-}

import Backend exposing (GrownPolycube)


describe : GrownPolycube -> String
describe grownPolycube =
    let
        boxVolume =
            List.product grownPolycube.boundingBox

        filledPercentage =
            if boxVolume == 0 then
                0

            else
                round (100 * toFloat (List.length grownPolycube.cells) / toFloat boxVolume)
    in
    "Boîte englobante "
        ++ String.join " × " (List.map String.fromInt grownPolycube.boundingBox)
        ++ ", remplie à "
        ++ String.fromInt filledPercentage
        ++ " %"
