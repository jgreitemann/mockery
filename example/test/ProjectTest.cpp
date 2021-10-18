#include <gtest/gtest.h>
#include <gmock/gmock.h>

#include <project/Project.h>

#include "ProjectStorageMock.h"

using ::testing::_;
using ::testing::NiceMock;
using ::testing::Return;

struct ProjectTest : ::testing::Test {
    auto OpenProject(std::string_view name) -> Project {
      if (!m_StorageMockUPtr)
        throw std::runtime_error{"Only one project can be opened per test case."};
      return Project{name, std::move(m_StorageMockUPtr)};
    }

    void SetUp() {
      ON_CALL(m_StorageMock, Open(_)).WillByDefault(Return(true));
    }

private:
    std::unique_ptr<NiceMock<ProjectStorageMock>> m_StorageMockUPtr = std::make_unique<NiceMock<ProjectStorageMock>>();

protected:
    NiceMock<ProjectStorageMock> &m_StorageMock = *m_StorageMockUPtr.get();
};

TEST_F(ProjectTest, Opening_a_project_which_does_not_exist_throws) {
  EXPECT_CALL(m_StorageMock, Open("/projects/my_project")).WillOnce(Return(false));

  EXPECT_THROW(OpenProject("my_project"), ProjectNotFound);
}

TEST_F(ProjectTest, Opening_a_project_which_already_exists_checks_for_settings_file_and_parses_it_if_it_exists) {
  constexpr int NonDefaultVerbosity = 3;
  static_assert(NonDefaultVerbosity != Project::DefaultVerbosity);

  EXPECT_CALL(m_StorageMock, Open("/projects/my_project"));
  EXPECT_CALL(m_StorageMock, List()).WillOnce(
          Return(std::map<std::string, Type>{{".git", Type::Directory},
                                             {".gitignore", Type::File},
                                             {"settings.toml", Type::File}}));
  EXPECT_CALL(m_StorageMock, ReadFile(Project::SettingsFilename))
          .WillOnce(Return("verbose=" + std::to_string(NonDefaultVerbosity)));
  EXPECT_CALL(m_StorageMock, Close());

  auto proj = OpenProject("my_project");
  EXPECT_EQ(proj.GetVerbosity(), NonDefaultVerbosity);
}

TEST_F(ProjectTest,
       Opening_a_project_which_already_exists_checks_for_settings_file_and_creates_a_default_one_if_it_does_not_exist) {
  EXPECT_CALL(m_StorageMock, Open("/projects/my_project"));
  EXPECT_CALL(m_StorageMock, List()).WillOnce(Return(std::map<std::string, Type>{}));

  auto proj = OpenProject("my_project");
  EXPECT_EQ(proj.GetVerbosity(), Project::DefaultVerbosity);

  ::testing::Mock::VerifyAndClearExpectations(&m_StorageMock);

  EXPECT_CALL(m_StorageMock, SaveFile(Project::SettingsFilename,
                                      "verbose=" + std::to_string(Project::DefaultVerbosity)));
  EXPECT_CALL(m_StorageMock, Close());
}

TEST_F(ProjectTest, Settings_are_written_prior_to_closing_the_project_when_they_have_been_changed) {
  EXPECT_CALL(m_StorageMock, Open("/projects/my_project"));
  EXPECT_CALL(m_StorageMock, List()).WillOnce(
          Return(std::map<std::string, Type>{{".git", Type::Directory},
                                             {".gitignore", Type::File},
                                             {"settings.toml", Type::File}}));
  EXPECT_CALL(m_StorageMock, ReadFile(Project::SettingsFilename)).WillOnce(Return("verbose=3"));

  auto proj = OpenProject("my_project");
  proj.SetVerbosity(2);
  ASSERT_EQ(proj.GetVerbosity(), 2);

  ::testing::Mock::VerifyAndClearExpectations(&m_StorageMock);

  EXPECT_CALL(m_StorageMock, SaveFile(Project::SettingsFilename, "verbose=2"));
  EXPECT_CALL(m_StorageMock, Close());
}
