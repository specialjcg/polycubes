module Page.Remarkable exposing (Model, Msg, init, start, subscriptions, update, view)

{-| Exceptional polycubes of a size, found by visiting them all: those with
the most loops, those whose loops avoid the 2 × 2 square, and the trees
fitting in the smallest box. Shapes that differ by a rotation are shown
once.
-}

import Backend exposing (RemarkablePolycubes, RemarkableShape, ShapeFamily)
import CellCountSlider
import Compactness
import Html exposing (Html, div, h2, p, section, text)
import Html.Attributes as Attr
import Scene


{-| Must match `MAX_REMARKABLE_CELL_COUNT` of the backend.
-}
largestSearchedCellCount : Int
largestSearchedCellCount =
    9


initialCellCount : Int
initialCellCount =
    8


type alias Model =
    { cellCount : Int
    , remarkable : Search
    }


type Search
    = Searching
    | Found RemarkablePolycubes
    | Failed String


type Msg
    = CellCountChosen String
    | SearchFinished (Result String RemarkablePolycubes)


init : Model
init =
    { cellCount = initialCellCount, remarkable = Searching }


{-| First search, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestSearch model.cellCount


requestSearch : Int -> ( Model, Cmd Msg )
requestSearch cellCount =
    ( { cellCount = cellCount, remarkable = Searching }
    , Backend.requestRemarkable cellCount
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestSearch (CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount)

        SearchFinished (Ok remarkable) ->
            ( { model | remarkable = Found remarkable }, Cmd.none )

        SearchFinished (Err reason) ->
            ( { model | remarkable = Failed reason }, Cmd.none )


subscriptions : Sub Msg
subscriptions =
    Backend.remarkableFinished SearchFinished


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view largestSearchedCellCount CellCountChosen model.cellCount ]
        , viewSearch model.remarkable
        ]


viewSearch : Search -> Html Msg
viewSearch search =
    case search of
        Searching ->
            p [ Attr.class "status" ] [ text "Recherche parmi tous les polycubes…" ]

        Found remarkable ->
            div []
                [ viewFamily "Le plus de boucles" (describeMostLooped remarkable.mostLooped) remarkable.mostLooped
                , viewSquareFreeLoops remarkable.squareFreeLoops
                , viewFamily "Arbres les plus ramassés"
                    "Sans aucune boucle, dans la boîte au plus petit grand côté, puis au plus petit volume."
                    remarkable.tightestTrees
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Recherche impossible : " ++ reason) ]


squareFreeTitle : String
squareFreeTitle =
    "Boucles sans carré 2 × 2"


viewSquareFreeLoops : Maybe ShapeFamily -> Html Msg
viewSquareFreeLoops squareFreeLoops =
    case squareFreeLoops of
        Just family ->
            viewFamily squareFreeTitle
                ("Le plus de boucles sans que quatre cubes forment un carré : "
                    ++ String.fromInt family.loopCount
                    ++ ". La boucle fait alors le tour d'un vide."
                )
                family

        Nothing ->
            section []
                [ h2 [] [ text squareFreeTitle ]
                , p [ Attr.class "explanation" ]
                    [ text "Aucune à cette taille : il faut au moins 6 cubes pour boucler sans carré." ]
                ]


describeMostLooped : ShapeFamily -> String
describeMostLooped family =
    if family.loopCount == 0 then
        "À cette taille aucun polycube n'a de boucle : toutes les formes sont à égalité."

    else
        String.fromInt family.loopCount
            ++ (if family.loopCount == 1 then
                    " boucle"

                else
                    " boucles"
               )
            ++ " au plus à cette taille, soit "
            ++ String.fromInt family.exposedFaceCount
            ++ " faces visibles seulement."


viewFamily : String -> String -> ShapeFamily -> Html Msg
viewFamily title explanation family =
    section []
        [ h2 [] [ text title ]
        , p [ Attr.class "explanation" ] [ text (explanation ++ " " ++ describeCounts family) ]
        , div [ Attr.class "gallery remarkable" ] (List.map viewShape family.shown)
        ]


describeCounts : ShapeFamily -> String
describeCounts family =
    let
        shownCount =
            List.length family.shown

        shapes =
            if family.shapeCount == 1 then
                "Une seule forme"

            else
                String.fromInt family.shapeCount ++ " formes à rotation près"

        hidden =
            if shownCount < family.shapeCount then
                " ; les " ++ String.fromInt shownCount ++ " plus symétriques sont montrées"

            else
                ""
    in
    shapes ++ ", " ++ describeFixedCount family.fixedCount ++ hidden ++ "."


describeFixedCount : Int -> String
describeFixedCount fixedCount =
    if fixedCount == 1 then
        "1 polycube fixe"

    else
        String.fromInt fixedCount ++ " polycubes fixes"


viewShape : RemarkableShape -> Html Msg
viewShape shape =
    section []
        [ Scene.view shape.polycube.cells
        , p [ Attr.class "status" ]
            [ text
                (describeOrientations shape.orientationCount
                    ++ ". "
                    ++ Compactness.describe shape.polycube
                )
            ]
        ]


describeOrientations : Int -> String
describeOrientations orientationCount =
    if orientationCount == 1 then
        "1 orientation"

    else
        String.fromInt orientationCount ++ " orientations"
