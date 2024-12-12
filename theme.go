package main

type ResourceType int8

const (
	SimpleText ResourceType = iota + 1
	RichText
	MultimediaText
	Interactive
)

type DifficultyLevel int8

const (
	Easy DifficultyLevel = iota + 1
	Medium
	Hard
)

type Question struct {
	Id                   int
	QuestionResourceType ResourceType
	ResourceURI          string
}

type Difficulty struct {
	Id                            DifficultyLevel
	TotalNumOfQuestions           int
	Questions                     []int
	ProportionOfQuestionsPerMatch int8
}

type Subtheme struct {
	Id           int
	Difficulties [3]DifficultyLevel
}

type Theme struct {
	Id        int
	Subthemes []int
}
