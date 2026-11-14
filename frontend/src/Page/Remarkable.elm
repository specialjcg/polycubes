module Page.Remarkable exposing (Model, Msg, init, start, subscriptions, update, view)

{-| The polycubes of a size that hold the most loops: the exceptions at the
far end of the count by number of loops. Shapes that differ by a rotation
are shown once.
-}

import Backend exposing (MostLoopedPolycubes, RemarkableShape)
import CellCountSlider
import Compactness
import Html exposing (Html, div, p, section, text)
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
    , mostLooped : Search
    }


type Search
    = Searching
    | Found MostLoopedPolycubes
    | Failed String


type Msg
    = CellCountChosen String
    | SearchFinished (Result String MostLoopedPolycubes)


init : Model
init =
    { cellCount = initialCellCount, mostLooped = Searching }


{-| First search, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestSearch model.cellCount


requestSearch : Int -> ( Model, Cmd Msg )
requestSearch cellCount =
    ( { cellCount = cellCount, mostLooped = Searching }
    , Backend.requestMostLooped cellCount
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestSearch (CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount)

        SearchFinished (Ok mostLooped) ->
            ( { model | mostLooped = Found mostLooped }, Cmd.none )

        SearchFinished (Err reason) ->
            ( { model | mostLooped = Failed reason }, Cmd.none )


subscriptions : Sub Msg
subscriptions =
    Backend.mostLoopedFinished SearchFinished


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view largestSearchedCellCount CellCountChosen model.cellCount ]
        , viewSearch model.mostLooped
        ]


viewSearch : Search -> Html Msg
viewSearch search =
    case search of
        Searching ->
            p [ Attr.class "status" ] [ text "Recherche parmi tous les polycubes…" ]

        Found mostLooped ->
            div []
                [ p [ Attr.class "explanation" ] [ text (describe mostLooped) ]
                , div [ Attr.class "gallery remarkable" ] (List.map viewShape mostLooped.shapes)
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Recherche impossible : " ++ reason) ]


describe : MostLoopedPolycubes -> String
describe mostLooped =
    let
        fixedCount =
            List.map .orientationCount mostLooped.shapes |> List.sum
    in
    if mostLooped.loopCount == 0 then
        "À cette taille aucun polycube n'a de boucle : voici toutes les formes, à rotation près."

    else
        "Le plus de boucles possible à cette taille : "
            ++ String.fromInt mostLooped.loopCount
            ++ ", soit "
            ++ String.fromInt mostLooped.exposedFaceCount
            ++ " faces visibles seulement. "
            ++ (if fixedCount == 1 then
                    "Un seul polycube y parvient."

                else
                    String.fromInt fixedCount
                        ++ " polycubes fixes y parviennent, qui se ramènent par rotation aux formes ci-dessous."
               )


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
